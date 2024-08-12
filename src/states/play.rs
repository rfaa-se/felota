use std::collections::{BTreeSet, HashMap, HashSet};

use raylib::prelude::*;

use crate::{
    bus::Bus,
    commands::{Command, EntityCommands},
    components::Generation,
    constants::{
        DEBUG_COLOR, HUD_BACKGROUND_COLOR, HUD_HEIGHT, RENDER_HEIGHT, RENDER_WIDTH, TICK_SCHEDULED,
    },
    entities::{Entities, Entity},
    forge::Forge,
    logic::Logic,
    messages::{Message, NetMessage, NetRequestMessage},
    render::Renderer,
};

pub struct Play {
    tick: u32,
    synchronized: bool,
    stalling: bool,
    network_data: NetworkData,
    player_data: PlayerData,
    camera: Camera2D,
    camera_target: Generation<Vector2>,
    entities: Entities,
    forge: Forge,
    logic: Logic,
    renderer: Renderer,
    commands: Vec<TickCommands>,
    command_queue: HashSet<Command>,
    actions: BTreeSet<Action>,
}

struct TickCommands {
    ready: bool,
    commands: Vec<EntityCommands>,
}

struct NetworkData {
    client_id: u32,
    client_ids: Vec<u32>,
    seed: u32,
}

struct PlayerData {
    player_id: usize,
    player_ids: Vec<usize>,
    map: HashMap<u32, usize>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Command(Command),
}

impl Play {
    pub fn new() -> Self {
        Self {
            tick: 0,
            synchronized: false,
            stalling: false,
            network_data: NetworkData {
                client_id: 0,
                client_ids: Vec::new(),
                seed: 0,
            },
            player_data: PlayerData {
                player_id: 0,
                player_ids: Vec::new(),
                map: HashMap::new(),
            },
            camera: Camera2D {
                offset: Vector2 {
                    x: (RENDER_WIDTH / 2) as f32,
                    y: ((RENDER_HEIGHT / 2) - HUD_HEIGHT / 2) as f32,
                },
                target: Vector2::zero(),
                rotation: 0.0,
                zoom: 1.0,
            },
            camera_target: Generation {
                old: Vector2::zero(),
                new: Vector2::zero(),
            },
            entities: Entities::new(),
            forge: Forge::new(),
            logic: Logic::new(),
            renderer: Renderer::new(),
            commands: Vec::new(),
            command_queue: HashSet::new(),
            actions: BTreeSet::new(),
        }
    }

    pub fn init(&mut self, bus: &mut Bus) {
        // we must synchronize to get all clients, local client, and rng seed
        bus.send(NetRequestMessage::Synchronize);
    }

    pub fn exit(&mut self) {
        self.tick = 0;
        self.synchronized = false;
        self.commands.clear();
        self.command_queue.clear();
    }

    pub fn update(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(bus);

        if !self.synchronized {
            return;
        }

        let tick_commands = &self.commands[self.tick as usize];

        self.stalling = !tick_commands.ready;

        if self.stalling {
            return;
        }

        self.logic.update(
            bus,
            &mut self.entities,
            &tick_commands.commands,
            &self.forge,
            h,
        );

        // the player centroid is used to set the camera target,
        // we want the player entity to be positioned in the middle of the screen,
        // this is why the camera is created with an offset and we only update the target
        if let Some(centroid) = self.entities.centroid(self.player_data.player_id) {
            self.camera_target = centroid;
        }

        // send the current command queue
        bus.send(NetRequestMessage::Commands(
            self.tick + TICK_SCHEDULED,
            self.command_queue.drain().collect(),
        ));

        // make sure we can receive the new commands
        self.commands.push(TickCommands {
            ready: false,
            commands: Vec::new(),
        });

        self.tick += 1;
    }

    pub fn input(&mut self, h: &mut RaylibHandle) {
        if h.is_key_down(KeyboardKey::KEY_LEFT) {
            self.actions.insert(Action::Command(Command::RotateLeft));
        }

        if h.is_key_down(KeyboardKey::KEY_RIGHT) {
            self.actions.insert(Action::Command(Command::RotateRight));
        }

        if h.is_key_down(KeyboardKey::KEY_UP) {
            self.actions.insert(Action::Command(Command::Accelerate));
        }

        if h.is_key_down(KeyboardKey::KEY_DOWN) {
            self.actions.insert(Action::Command(Command::Decelerate));
        }

        if h.is_key_down(KeyboardKey::KEY_SPACE) {
            self.actions.insert(Action::Command(Command::Projectile));
        }
    }

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, delta: f32) {
        if !self.synchronized {
            return;
        }

        // make the camera follow the player
        self.camera.target = self.camera_target.old.lerp(self.camera_target.new, delta);

        {
            let mut r = r.begin_mode2D(self.camera);

            // the viewport is used to cull entities not currently shown on screen
            // TODO: should not be smaller than the screen,
            // only smaller now to make sure code works when culling is implemented
            let viewport = Rectangle {
                x: self.camera.target.x - self.camera.offset.x + 100.0,
                y: self.camera.target.y - self.camera.offset.y + 100.0,
                width: self.camera.offset.x * 2.0 - 200.0,
                height: self.camera.offset.y * 2.0 - 200.0,
            };

            self.renderer.draw(&mut r, &self.entities, viewport, delta);
        }

        if self.stalling {
            let len = r.measure_text("stalling", 10);
            r.draw_text(
                "stalling",
                RENDER_WIDTH as i32 / 2 - len / 2,
                RENDER_HEIGHT / 2,
                10,
                DEBUG_COLOR,
            );
        }

        self.draw_hud(r, delta);

        r.draw_text(&format!("tick {}", self.tick), 3, 22, 10, DEBUG_COLOR);
        r.draw_text(
            &format!("ents {}", self.entities.total()),
            3,
            32,
            10,
            DEBUG_COLOR,
        );
    }

    pub fn message(&mut self, msg: &Message) {
        match msg {
            Message::Net(msg) => match msg {
                NetMessage::Synchronize(seed, cid, cids) => {
                    // set the networking data
                    self.network_data.seed = *seed;
                    self.network_data.client_id = *cid;
                    self.network_data.client_ids = cids.to_vec();

                    // create the players in the cosmos and set the player data
                    for client_id in cids.iter() {
                        let player_id = self.add_player();

                        self.player_data.player_ids.push(player_id);
                        self.player_data.map.insert(*client_id, player_id);

                        if client_id == cid {
                            self.player_data.player_id = player_id;
                        }
                    }

                    // commands are scheduled x ticks in the future,
                    // make sure we can progress for the first ticks
                    for _ in 0..TICK_SCHEDULED {
                        self.commands.push(TickCommands {
                            ready: true,
                            commands: Vec::new(),
                        });
                    }

                    // we are now fully synced and can begin playing!
                    self.synchronized = true;
                }
                NetMessage::Commands(cid, tick, cmds) => {
                    let tick_commands = &mut self.commands[*tick as usize];

                    // add the client's commands
                    tick_commands.commands.push(EntityCommands {
                        id: self.player_data.map[cid],
                        commands: cmds.clone(),
                    });

                    // if there are as many entity commands as there are players,
                    // then we have received everything and are ready to progress
                    tick_commands.ready =
                        tick_commands.commands.len() == self.player_data.player_ids.len();
                }
                _ => return,
            },
            _ => return,
        };
    }

    fn draw_hud(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, _delta: f32) {
        r.draw_line(
            0,
            RENDER_HEIGHT - HUD_HEIGHT,
            RENDER_WIDTH,
            RENDER_HEIGHT - HUD_HEIGHT,
            HUD_BACKGROUND_COLOR,
        );
    }

    fn add_player(&mut self) -> usize {
        let entity = Entity::Triship(self.forge.triship());
        self.entities.add(entity)
    }

    fn action(&mut self, _bus: &mut Bus) {
        while let Some(action) = self.actions.pop_last() {
            match action {
                Action::Command(cmd) => {
                    self.command_queue.insert(cmd);
                }
            }
        }
    }
}
