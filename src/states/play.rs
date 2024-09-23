use std::collections::{BTreeSet, HashMap};

use raylib::prelude::*;

use crate::{
    bus::Bus,
    commands::{Command, EntityCommands},
    components::Generation,
    constants::{
        DEBUG_COLOR, HUD_BACKGROUND_COLOR, HUD_HEIGHT, RENDER_HEIGHT, RENDER_WIDTH, RESPAWN_TIMER,
        TICK_SCHEDULED,
    },
    entities::{Entities, Entity, EntityIndex},
    forge::Forge,
    logic::Logic,
    messages::{
        EngineMessage, EngineRequestMessage, LogicMessage, Message, NetMessage, NetRequestMessage,
    },
    render::Renderer,
};

pub struct Play {
    tick: u32,
    synchronized: bool,
    debug: bool,
    stalling: bool,
    paused: bool,
    network_data: NetworkData,
    player_data: PlayerData,
    camera: Camera2D,
    camera_target: Generation<Vector2>,
    entities: Entities,
    forge: Forge,
    logic: Logic,
    renderer: Renderer,
    commands: Vec<TickCommands>,
    command_queue: BTreeSet<Command>,
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
    player_entity_id: usize,
    entity_ids: Vec<usize>,
    map: HashMap<u32, usize>,
    respawn_timers: Vec<(usize, u8)>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Synchronize(u32, u32, Vec<u32>),
    Command(Command),
    ToggleInterpolation,
    ToggleDebug,
    TogglePause,
    SpawnTriship(i32, i32),
}

impl Play {
    pub fn new() -> Self {
        Self {
            tick: 0,
            synchronized: false,
            debug: false,
            stalling: false,
            paused: false,
            network_data: NetworkData {
                client_id: 0,
                client_ids: Vec::new(),
                seed: 0,
            },
            player_data: PlayerData {
                player_entity_id: 0,
                entity_ids: Vec::new(),
                map: HashMap::new(),
                respawn_timers: Vec::new(),
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
            command_queue: BTreeSet::new(),
            actions: BTreeSet::new(),
        }
    }

    pub fn init(&mut self, bus: &mut Bus) {
        // we must synchronize to get current options
        bus.send(EngineRequestMessage::Synchronize);

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
        self.action(bus, h);

        if !self.synchronized || self.paused {
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

        self.update_respawns();

        // the player centroid is used to set the camera target,
        // we want the player entity to be positioned in the middle of the screen,
        // this is why the camera is created with an offset and we only update the target
        if let Some(centroid) = self.entities.centroid(self.player_data.player_entity_id) {
            self.camera_target = centroid;
        }

        let mut q = Vec::new();
        while let Some(c) = self.command_queue.pop_first() {
            q.push(c);
        }

        // send the current command queue
        bus.send(NetRequestMessage::Commands(
            self.tick + TICK_SCHEDULED,
            q.into_boxed_slice(),
        ));

        // make sure we can receive the new commands
        self.commands.push(TickCommands {
            ready: false,
            commands: Vec::new(),
        });

        self.tick += 1;
    }

    pub fn input(&mut self, h: &mut RaylibHandle) {
        // options
        if h.is_key_pressed(KeyboardKey::KEY_F1) {
            self.actions.insert(Action::ToggleInterpolation);
        }

        if h.is_key_pressed(KeyboardKey::KEY_F2) {
            self.actions.insert(Action::ToggleDebug);
        }

        if h.is_key_pressed(KeyboardKey::KEY_F3) {
            self.actions.insert(Action::TogglePause);
        }

        if h.is_key_pressed(KeyboardKey::KEY_F4) {
            let pos = h.get_screen_to_world2D(h.get_mouse_position(), self.camera);
            self.actions
                .insert(Action::SpawnTriship(pos.x as i32, pos.y as i32));
        }

        // gameplay
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

        if h.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            self.actions.insert(Action::Command(Command::Boost));
        }

        if h.is_key_down(KeyboardKey::KEY_LEFT_CONTROL) {
            self.actions.insert(Action::Command(Command::Torpedo));
        }
    }

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, delta: f32) {
        if !self.synchronized {
            return;
        }

        let delta = if self.stalling || self.paused {
            1.0
        } else {
            delta
        };

        // make the camera follow the player
        self.camera.target = self.camera_target.old.lerp(self.camera_target.new, delta);

        {
            let mut r = r.begin_mode2D(self.camera);

            // the viewport is used to cull entities not currently shown on screen
            // TODO: should not be smaller than the screen,
            // only smaller now to make sure code works when culling is implemented
            let viewport = Rectangle {
                x: self.camera.target.x - self.camera.offset.x,
                y: self.camera.target.y - self.camera.offset.y,
                width: self.camera.offset.x * 2.0,
                height: self.camera.offset.y * 2.0,
                // x: self.camera.target.x - self.camera.offset.x + 100.0,
                // y: self.camera.target.y - self.camera.offset.y + 100.0,
                // width: self.camera.offset.x * 2.0 - 200.0,
                // height: self.camera.offset.y * 2.0 - 200.0,
            };

            if self.debug {
                self.logic.draw(&mut r);
            }

            self.renderer
                .draw(&mut r, &self.entities, viewport, self.debug, delta);
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

        if self.paused {
            let len = r.measure_text("paused", 10);
            r.draw_text(
                "paused",
                RENDER_WIDTH as i32 / 2 - len / 2,
                RENDER_HEIGHT / 2,
                10,
                DEBUG_COLOR,
            );
        }

        self.draw_hud(r, delta);

        if self.debug {
            let mouse_screen = r.get_mouse_position();
            let mouse_world = r.get_screen_to_world2D(mouse_screen, self.camera);
            r.draw_text(
                &format!("{}, {}", mouse_world.x, mouse_world.y),
                mouse_screen.x as i32 + 5,
                mouse_screen.y as i32 - 12,
                10,
                DEBUG_COLOR,
            );
        }

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
                    self.actions
                        .insert(Action::Synchronize(*seed, *cid, cids.to_vec()));
                }
                NetMessage::Commands(cid, tick, cmds) => {
                    // TODO: this might panic, investigate, make sure the index exists before we access it?
                    let tick_commands = &mut self.commands[*tick as usize];

                    // add the client's commands
                    tick_commands.commands.push(EntityCommands {
                        id: self.player_data.map[cid],
                        commands: cmds.clone(),
                    });

                    // if there are as many entity commands as there are players,
                    // then we have received everything and are ready to progress
                    tick_commands.ready =
                        tick_commands.commands.len() == self.player_data.entity_ids.len();
                }
                NetMessage::TogglePause(_cid) => {
                    // TODO: might be interesting to display who toggled pause
                    self.paused = !self.paused;
                }
                _ => return,
            },
            Message::Logic(LogicMessage::EntityDead(eid, eidx)) => match eidx {
                EntityIndex::Triship(_) => {
                    if self.player_data.entity_ids.contains(eid) {
                        // player has been killed, start a respwan timer
                        self.player_data.respawn_timers.push((*eid, RESPAWN_TIMER));

                        // if we have died, let's set the camera target to the latest known position
                        if self.player_data.player_entity_id == *eid {
                            self.camera_target = Generation {
                                old: self.camera_target.new,
                                new: self.camera_target.new,
                            };
                        }
                    }
                }
                _ => return,
            },
            Message::Engine(
                EngineMessage::Synchronize(debug) | EngineMessage::ToggleDebug(debug),
            ) => {
                self.debug = *debug;
            }
            _ => return,
        }
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

    fn action(&mut self, bus: &mut Bus, h: &mut RaylibHandle) {
        while let Some(action) = self.actions.pop_last() {
            match action {
                Action::Command(cmd) => {
                    self.command_queue.insert(cmd);
                }
                Action::ToggleInterpolation => {
                    bus.send(EngineRequestMessage::ToggleInterpolation);
                }
                Action::ToggleDebug => {
                    bus.send(EngineRequestMessage::ToggleDebug);
                }
                Action::Synchronize(seed, cid, cids) => {
                    // seed the stars
                    for star in self.forge.stars(h) {
                        self.entities.add(Entity::Star(star));
                    }

                    // create the players in the cosmos and set the player data
                    for client_id in cids.iter() {
                        let entity = Entity::Triship(self.forge.triship(Vector2::new(25.0, 25.0)));
                        let eid = self.entities.add(entity);

                        self.player_data.entity_ids.push(eid);
                        self.player_data.map.insert(*client_id, eid);

                        if *client_id == cid {
                            self.player_data.player_entity_id = eid;
                        }
                    }

                    // commands are scheduled x ticks in the future,
                    // make sure we can progress for the first ticks
                    for _ in 0..=TICK_SCHEDULED {
                        self.commands.push(TickCommands {
                            ready: true,
                            commands: Vec::new(),
                        });
                    }

                    // set the networking data
                    self.network_data.seed = seed;
                    self.network_data.client_id = cid;
                    self.network_data.client_ids = cids;

                    // we are now fully synced and can begin playing!
                    self.synchronized = true;
                }
                Action::TogglePause => {
                    bus.send(NetRequestMessage::TogglePause);
                }
                Action::SpawnTriship(x, y) => {
                    let entity =
                        Entity::Triship(self.forge.triship(Vector2::new(x as f32, y as f32)));
                    self.entities.add(entity);
                }
            }
        }
    }

    fn update_respawns(&mut self) {
        self.player_data.respawn_timers.retain_mut(|(eid, timer)| {
            *timer -= 1;

            if *timer > 0 {
                return true;
            }

            let entity = self.forge.triship(Vector2::new(25.0, 25.0));
            let new_eid = self.entities.add(Entity::Triship(entity));

            if self.player_data.player_entity_id == *eid {
                self.player_data.player_entity_id = new_eid;
            }

            for (_, entity_id) in self.player_data.map.iter_mut() {
                if entity_id == eid {
                    *entity_id = new_eid;
                }
            }

            self.player_data.entity_ids.retain(|x| x != eid);
            self.player_data.entity_ids.push(new_eid);

            false
        });
    }
}
