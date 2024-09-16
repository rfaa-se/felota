use std::collections::BTreeSet;

use raylib::prelude::*;

use crate::{
    bus::Bus,
    constants::{DEBUG_COLOR, RENDER_HEIGHT, RENDER_WIDTH, WINDOW_HEIGHT, WINDOW_WIDTH},
    messages::{EngineMessage, EngineRequestMessage, Message, StateRequestMessage},
    states::State,
    systems::Systems,
};

pub struct Engine {
    handle: RaylibHandle,
    thread: RaylibThread,
    render_texture: RenderTexture2D,
    systems: Systems,
    bus: Bus,
}

pub struct System {
    tps_current: u32,
    tps_counter: u32,
    interpolate: bool,
    debug: bool,
    actions: BTreeSet<Action>,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    ToggleInterpolation,
    ToggleDebug,
    Synchronize,
}

impl Engine {
    pub fn new() -> Self {
        let (mut handle, thread) = raylib::init()
            .size(WINDOW_WIDTH, WINDOW_HEIGHT)
            .title("felota")
            .build();

        let render_texture =
            match handle.load_render_texture(&thread, RENDER_WIDTH as u32, RENDER_HEIGHT as u32) {
                Ok(render_texture) => render_texture,
                Err(error) => panic!("Could not create render texture: {}", error),
            };

        Self {
            handle,
            thread,
            render_texture,
            systems: Systems::new(),
            bus: Bus::new(),
        }
    }

    pub fn run(&mut self) {
        self.init();

        let h = &mut self.handle;
        let s = &mut self.systems;
        let b = &mut self.bus;
        let size = 1.0 / 16.0;
        let mut accumulator = 0.0;
        let mut timer = 0.0;

        // TODO: handle this in a better way
        while !h.window_should_close() {
            let t = h.get_frame_time();

            accumulator += t;
            timer += t;

            // deal with input as often as possible
            s.states.input(h);

            // update at a fixed time interval
            while accumulator > size {
                accumulator -= size;

                // update bus and all systems
                b.update(s);
                s.engine.update(h, b);
                s.net.update(h, b);
                s.logs.update(h, b);
                s.states.update(h, b);

                // measure ticks per second
                s.engine.tps_counter += 1;
                if timer >= 1.0 {
                    timer -= 1.0;
                    s.engine.tps_current = s.engine.tps_counter;
                    s.engine.tps_counter = 0;
                }

                // TODO: make sure we draw every now and then if we spend too much time in here
            }

            // draw as often as possible
            let mut d = h.begin_drawing(&self.thread);

            {
                // to be able to support different screen resolutions,
                // we draw everything to a render texture and then scale it
                let r = &mut d.begin_texture_mode(&self.thread, &mut self.render_texture);

                // delta is used to smooth movement while interpolating between old and new states
                let delta = if s.engine.interpolate {
                    accumulator / size
                } else {
                    1.0
                };

                // paint it black
                r.clear_background(Color::BLACK);

                // draw everything
                s.engine.draw(r, delta);
                s.states.draw(r, delta);
                s.logs.draw(r, delta);
            }

            // render texture must be y-flipped due to default OpenGL coordinates (left-bottom)
            d.draw_texture_pro(
                &self.render_texture,
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: self.render_texture.texture.width as f32,
                    height: -self.render_texture.texture.height as f32,
                },
                Rectangle {
                    x: 0.0,
                    y: 0.0,
                    width: d.get_screen_width() as f32,
                    height: d.get_screen_height() as f32,
                },
                Vector2 { x: 0.0, y: 0.0 },
                0.0,
                Color::WHITE,
            );
        }
    }

    fn init(&mut self) {
        self.bus.send(StateRequestMessage::Set(State::Menu));
    }
}

impl System {
    pub fn new() -> Self {
        Self {
            tps_current: 0,
            tps_counter: 0,
            interpolate: true,
            debug: true,
            actions: BTreeSet::new(),
        }
    }

    pub fn update(&mut self, _h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(bus);
    }

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, _delta: f32) {
        r.draw_text(&format!("fps {}", r.get_fps()), 3, 2, 10, DEBUG_COLOR);
        r.draw_text(&format!("tps {}", self.tps_current), 3, 12, 10, DEBUG_COLOR);
    }

    pub fn message(&mut self, msg: &Message) {
        match msg {
            Message::Engine(msg) => match msg {
                EngineMessage::Request(msg) => match msg {
                    EngineRequestMessage::ToggleInterpolation => {
                        self.actions.insert(Action::ToggleInterpolation);
                    }
                    EngineRequestMessage::ToggleDebug => {
                        self.actions.insert(Action::ToggleDebug);
                    }
                    EngineRequestMessage::Synchronize => {
                        self.actions.insert(Action::Synchronize);
                    }
                },
                _ => return,
            },
            _ => return,
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop_last() {
            match action {
                Action::ToggleInterpolation => {
                    self.interpolate = !self.interpolate;
                    bus.send(EngineMessage::ToggleInterpolation(self.interpolate));
                }
                Action::ToggleDebug => {
                    self.debug = !self.debug;
                    bus.send(EngineMessage::ToggleDebug(self.debug));
                }
                Action::Synchronize => {
                    bus.send(EngineMessage::Synchronize(self.debug));
                }
            }
        }
    }
}
