use std::collections::{BTreeSet, HashMap};

use raylib::prelude::*;

use crate::{
    bus::Bus,
    commands::{Command, EntityCommands, Spawn},
    components::{Centroidable, Generation},
    constants::{
        COSMOS_HEIGHT, COSMOS_WIDTH, DEBUG_COLOR, HUD_BACKGROUND_COLOR, HUD_HEIGHT,
        HUD_SEPARATOR_COLOR, HUD_WIDTH, HUD_X, HUD_Y, MINIMAP_AREA_HEIGHT, MINIMAP_AREA_WIDTH,
        MINIMAP_HEIGHT, MINIMAP_WIDTH, MINIMAP_X, MINIMAP_Y, RENDER_WIDTH, RESPAWN_TIMER,
        TICK_SCHEDULED, VIEWPORT_HEIGHT, VIEWPORT_WIDTH,
    },
    entities::{Entities, Entity, EntityIndex},
    forge::Forge,
    logic::Logic,
    messages::{
        EngineMessage, EngineRequestMessage, LogicMessage, Message, NetMessage, NetRequestMessage,
    },
    quadtree::QuadTree,
    render::Renderer,
    utils::minimap_translate,
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
    render_data: RenderData,
    quadtree: QuadTree,
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

pub struct RenderData {
    pub target: Option<usize>,
    pub target_timer: u8,
    pub target_eidx: Option<EntityIndex>,
    pub player_entity_id: usize,
    pub player_eidx: Option<EntityIndex>,
}

struct PlayerData {
    player_entity_id: usize,
    hud_data: HudData,
    entity_ids: Vec<usize>,
    map: HashMap<u32, usize>,
    respawn_timers: Vec<(usize, u8)>,
}

struct HudData {
    life: f32,
    speed: f32,
    boost_active: u8,
    boost_cooldown: u8,
    torpedo_cooldown: u8,
    target: Option<usize>,
    target_timer: u8,
    minimap_entities: Vec<(Vector2, f32, Color)>,
    minimap_xy: Vector2,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Synchronize(u32, u32, Vec<u32>),
    Command(Command),
    ToggleInterpolation,
    ToggleDebug,
    TogglePause,
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
                hud_data: HudData {
                    life: 0.0,
                    speed: 0.0,
                    boost_active: 0,
                    boost_cooldown: 0,
                    torpedo_cooldown: 0,
                    target: None,
                    target_timer: 0,
                    minimap_entities: Vec::new(),
                    minimap_xy: Vector2::zero(),
                },
                entity_ids: Vec::new(),
                map: HashMap::new(),
                respawn_timers: Vec::new(),
            },
            render_data: RenderData {
                target: None,
                target_timer: 0,
                target_eidx: None,
                player_entity_id: 0,
                player_eidx: None,
            },
            camera: Camera2D {
                offset: Vector2 {
                    x: (VIEWPORT_WIDTH / 2) as f32,
                    y: (VIEWPORT_HEIGHT / 2) as f32,
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
            quadtree: QuadTree::new(COSMOS_WIDTH, COSMOS_HEIGHT),
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
            &mut self.quadtree,
            h,
        );

        self.update_player_respawns();
        self.update_player_data();
        self.update_render_data();

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
                .insert(Action::Command(Command::Spawn(Spawn::Triship(
                    pos.x as i32,
                    pos.y as i32,
                ))));
        }

        // TODO: we want these bindings to be configurable.. in the future :)
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

        if h.is_key_pressed(KeyboardKey::KEY_TAB) {
            self.actions.insert(Action::Command(Command::TargetLock));
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
            let viewport = Rectangle {
                x: self.camera.target.x - self.camera.offset.x,
                y: self.camera.target.y - self.camera.offset.y,
                width: self.camera.offset.x * 2.0,
                height: self.camera.offset.y * 2.0,
            };

            // TODO: should we really render this here? renderer?
            if self.debug {
                self.quadtree.draw(&mut r);
            }

            self.renderer.draw(
                &mut r,
                &self.entities,
                &self.render_data,
                viewport,
                self.debug,
                delta,
            );
        }

        if self.stalling {
            let len = r.measure_text("stalling", 10);
            r.draw_text(
                "stalling",
                VIEWPORT_WIDTH as i32 / 2 - len / 2,
                100,
                10,
                DEBUG_COLOR,
            );
        }

        if self.paused {
            let len = r.measure_text("paused", 10);
            r.draw_text(
                "paused",
                RENDER_WIDTH as i32 / 2 - len / 2,
                100,
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

                        // if we have died, let's reset the player data
                        if self.player_data.player_entity_id == *eid {
                            self.reset_data();
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
        r.draw_rectangle(HUD_X, HUD_Y, HUD_WIDTH, HUD_HEIGHT, HUD_BACKGROUND_COLOR);
        r.draw_line(HUD_X, HUD_Y, HUD_X, HUD_Y + HUD_HEIGHT, HUD_SEPARATOR_COLOR);

        let pad_x = 8;
        let pad_y = 3;
        let data = &self.player_data.hud_data;

        r.draw_text(
            "LIFE",
            HUD_X + pad_x,
            HUD_Y + 10 + pad_y * 1,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &format!("{:.2}", data.life),
            HUD_X + 70 + pad_x,
            HUD_Y + 10 + pad_y * 1,
            10,
            DEBUG_COLOR,
        );

        r.draw_text(
            "SPEED",
            HUD_X + pad_x,
            HUD_Y + 20 + pad_y * 2,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &format!("{:.2}", data.speed),
            HUD_X + 70 + pad_x,
            HUD_Y + 20 + pad_y * 2,
            10,
            DEBUG_COLOR,
        );

        r.draw_text(
            "BOOST",
            HUD_X + pad_x,
            HUD_Y + 30 + pad_y * 3,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &format!("{} ", data.boost_active),
            HUD_X + 70 + pad_x,
            HUD_Y + 30 + pad_y * 3,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &format!("{} ", data.boost_cooldown),
            HUD_X + 70 + pad_x,
            HUD_Y + 30 + pad_y * 3,
            10,
            DEBUG_COLOR,
        );

        r.draw_text(
            "TORPEDO",
            HUD_X + pad_x,
            HUD_Y + 40 + pad_y * 4,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &format!("{}", data.torpedo_cooldown),
            HUD_X + 70 + pad_x,
            HUD_Y + 40 + pad_y * 4,
            10,
            DEBUG_COLOR,
        );

        r.draw_text(
            "TARGET",
            HUD_X + pad_x,
            HUD_Y + 50 + pad_y * 5,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &format!("{}", data.target_timer),
            HUD_X + 70 + pad_x,
            HUD_Y + 50 + pad_y * 5,
            10,
            DEBUG_COLOR,
        );
        r.draw_text(
            &match data.target {
                Some(target) => target.to_string(),
                None => "-".to_string(),
            },
            HUD_X + pad_x,
            HUD_Y + 60 + pad_y * 6,
            10,
            DEBUG_COLOR,
        );

        if data.target.is_some() && data.target_timer == 0 {
            r.draw_text(
                "LOCKED",
                HUD_X + pad_x + 70,
                HUD_Y + 60 + pad_y * 6,
                10,
                DEBUG_COLOR,
            );
        }

        // render minimap
        r.draw_rectangle_lines_ex(
            Rectangle {
                x: MINIMAP_X as f32 - 1.0,
                y: MINIMAP_Y as f32 - 1.0,
                width: MINIMAP_WIDTH as f32 + 2.0,
                height: MINIMAP_HEIGHT as f32 + 2.0,
            },
            1.0,
            HUD_SEPARATOR_COLOR,
        );

        r.draw_rectangle_rec(
            Rectangle {
                x: MINIMAP_X as f32,
                y: MINIMAP_Y as f32,
                width: MINIMAP_WIDTH as f32,
                height: MINIMAP_HEIGHT as f32,
            },
            Color::BLACK,
        );

        // render entities in minimap
        for (pos, size, color) in data.minimap_entities.iter() {
            r.draw_rectangle_v(pos, Vector2::new(*size, *size), color);
        }

        // render cosmos bounds in minimap
        let area = Rectangle {
            x: self.player_data.hud_data.minimap_xy.x - (MINIMAP_AREA_WIDTH / 2) as f32,
            y: self.player_data.hud_data.minimap_xy.y - (MINIMAP_AREA_HEIGHT / 2) as f32,
            width: MINIMAP_AREA_WIDTH as f32,
            height: MINIMAP_AREA_HEIGHT as f32,
        };

        let ay = area.y;
        let ax = area.x;
        let ayh = ay + area.height;
        let axw = ax + area.width;

        let x_top_left = ax.max(0.0);
        let y_top_left = ay.max(0.0);

        let x_top_right = axw.min(COSMOS_WIDTH as f32);
        let y_top_right = ay.max(0.0);

        let x_bottom_left = ax.max(0.0);
        let y_bottom_left = ayh.min(COSMOS_HEIGHT as f32);

        let x_bottom_right = axw.min(COSMOS_WIDTH as f32);
        let y_bottom_right = ayh.min(COSMOS_HEIGHT as f32);

        let minimap = self.player_data.hud_data.minimap_xy;

        // left border
        if ax < x_top_left && axw > x_top_left && y_top_left < y_bottom_left {
            let start = minimap_translate(x_top_left, y_top_left, minimap);
            let end = minimap_translate(x_top_left, y_bottom_left, minimap);

            r.draw_line_v(start, end, Color::RED);
        }

        // right border
        if axw > x_top_right && ax < x_top_right && y_top_right < y_bottom_right {
            let start = minimap_translate(x_top_right, y_top_right, minimap);
            let end = minimap_translate(x_bottom_right, y_bottom_right, minimap);

            r.draw_line_v(start, end, Color::RED);
        }

        // top border
        if ay < y_top_left && ayh > y_top_left && x_top_left < x_top_right {
            let start = minimap_translate(x_top_left, y_top_left, minimap);
            let end = minimap_translate(x_top_right, y_top_right, minimap);

            r.draw_line_v(start, end, Color::RED);
        }

        // bottom border
        if ayh > y_bottom_left && ay < y_bottom_right && x_bottom_left < x_bottom_right {
            let start = minimap_translate(x_bottom_left, y_bottom_left, minimap);
            let end = minimap_translate(x_bottom_right, y_bottom_right, minimap);

            r.draw_line_v(start, end, Color::RED);
        }
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
            }
        }
    }

    fn update_player_respawns(&mut self) {
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

    fn update_player_data(&mut self) {
        let Some(eidx) = self.entities.entity(self.player_data.player_entity_id) else {
            return;
        };

        let e = match eidx {
            EntityIndex::Triship(idx) => &self.entities.triships[idx].entity,
            _ => return,
        };

        // the player centroid is used to set the camera target,
        // we want the player entity to be positioned in the middle of the screen,
        // this is why the camera is created with an offset and we only update the target
        self.camera_target.old = e.body.state.old.shape.centroid();
        self.camera_target.new = e.body.state.new.shape.centroid();

        let hud = &mut self.player_data.hud_data;

        hud.life = e.life;
        hud.speed = e.motion.velocity.length();
        hud.boost_active = if e.boost.active {
            e.boost.lifetime.current
        } else {
            0
        };

        hud.boost_cooldown = if e.boost.active {
            e.boost.cooldown.current
        } else {
            0
        };

        hud.torpedo_cooldown = e.cooldown_torpedo.current;

        hud.target = e.targeting.eid;

        if e.targeting.timer.current != e.targeting.timer.max {
            hud.target_timer = e.targeting.timer.current;
        } else {
            hud.target_timer = 0;
        }

        hud.minimap_xy = self.camera_target.new;
        hud.minimap_entities = self
            .quadtree
            .get(
                &Rectangle {
                    x: hud.minimap_xy.x - (MINIMAP_AREA_WIDTH / 2) as f32,
                    y: hud.minimap_xy.y - (MINIMAP_AREA_HEIGHT / 2) as f32,
                    width: MINIMAP_AREA_WIDTH as f32,
                    height: MINIMAP_AREA_HEIGHT as f32,
                },
                &self.entities,
            )
            .iter()
            .filter_map(|eidx_rnd| match eidx_rnd {
                EntityIndex::Triship(idx) => Some((
                    self.entities.triships[*idx]
                        .entity
                        .body
                        .state
                        .new
                        .shape
                        .centroid(),
                    4.0,
                    if *eidx_rnd == eidx {
                        Color::WHITESMOKE
                    } else {
                        Color::RED
                    },
                )),
                EntityIndex::Torpedo(idx) => Some((
                    self.entities.torpedoes[*idx]
                        .entity
                        .body
                        .state
                        .new
                        .shape
                        .centroid(),
                    2.0,
                    Color::RED,
                )),
                _ => None,
            })
            .map(|(centroid, size, color)| {
                (
                    minimap_translate(centroid.x, centroid.y, hud.minimap_xy),
                    size,
                    color,
                )
            })
            .collect();
    }

    fn update_render_data(&mut self) {
        let p = &self.player_data;
        let r = &mut self.render_data;

        r.target = p.hud_data.target;
        r.target_timer = p.hud_data.target_timer;

        r.target_eidx = match r.target {
            Some(eid) => self.entities.entity(eid),
            None => None,
        };

        r.player_entity_id = p.player_entity_id;
        r.player_eidx = self.entities.entity(r.player_entity_id);
    }

    fn reset_data(&mut self) {
        let r = &mut self.render_data;
        let p = &mut self.player_data;

        r.target = None;
        r.target_timer = 0;
        r.target_eidx = None;
        r.player_entity_id = 0;
        r.player_eidx = None;

        // set the camera target to the latest known position
        self.camera_target = Generation {
            old: self.camera_target.new,
            new: self.camera_target.new,
        };

        p.hud_data.life = 0.0;
        p.hud_data.speed = 0.0;
        p.hud_data.boost_cooldown = 0;
        p.hud_data.torpedo_cooldown = 0;
    }
}
