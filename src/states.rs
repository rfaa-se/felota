pub mod play;

use raylib::prelude::*;

use crate::{
    bus::Bus,
    messages::{Message, StateMessage, StateRequestMessage},
};

use self::play::Play;

#[derive(Copy, Clone, Debug)]
pub enum State {
    None,
    Play,
}

enum Action {
    Set(State),
}

struct States {
    play: Play,
}

pub struct System {
    current: State,
    states: States,
    actions: Vec<Action>,
}

impl System {
    pub fn new() -> Self {
        Self {
            current: State::None,
            states: States { play: Play::new() },
            actions: Vec::new(),
        }
    }

    pub fn update(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(bus);

        match self.current {
            State::None => (),
            State::Play => self.states.play.update(h, bus),
        }
    }

    pub fn input(&mut self, h: &mut RaylibHandle) {
        match self.current {
            State::None => (),
            State::Play => self.states.play.input(h),
        }
    }

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, delta: f32) {
        match self.current {
            State::None => (),
            State::Play => self.states.play.draw(r, delta),
        }
    }

    pub fn message(&mut self, msg: &Message) {
        if let Message::State(StateMessage::Request(req)) = msg {
            match req {
                StateRequestMessage::Set(state) => {
                    self.actions.push(Action::Set(*state));
                }
            }
        }

        match self.current {
            State::None => (),
            State::Play => self.states.play.message(msg),
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Set(state) => {
                    bus.send(StateMessage::Set(state));

                    // exit the old state
                    match self.current {
                        State::None => (),
                        State::Play => self.states.play.exit(),
                    }

                    self.current = state;

                    // init the new state
                    match self.current {
                        State::None => (),
                        State::Play => self.states.play.init(bus),
                    }
                }
            }
        }
    }
}
