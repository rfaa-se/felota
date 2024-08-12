use std::collections::BTreeSet;

use raylib::prelude::*;

use crate::{
    bus::Bus,
    constants::{DEBUG_COLOR, RENDER_WIDTH},
    messages::{Message, NetMessage, NetRequestMessage, StateRequestMessage},
};

use super::State;

pub struct Menu {
    actions: BTreeSet<Action>,
    hosting: bool,
    hosted: bool,
    joining: bool,
    host_text: String,
    join_text: String,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Host,
    Join(String),
    Hosted,
    Connected,
    Disconnected,
}

impl Menu {
    pub fn new() -> Self {
        Self {
            actions: BTreeSet::new(),
            hosting: false,
            hosted: false,
            joining: false,
            host_text: "[h]ost".to_owned(),
            join_text: "[j]oin".to_owned(),
        }
    }

    pub fn init(&mut self, _bus: &mut Bus) {}

    pub fn exit(&mut self) {
        self.hosting = false;
        self.joining = false;
        self.hosted = false;
    }

    pub fn update(&mut self, _h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(bus);
    }

    pub fn input(&mut self, h: &mut RaylibHandle) {
        if h.is_key_pressed(KeyboardKey::KEY_H) && !self.hosting && !self.hosted {
            self.actions.insert(Action::Host);
        }

        if h.is_key_pressed(KeyboardKey::KEY_J) && !self.joining {
            self.actions.insert(Action::Join("127.0.0.1".to_owned()));
        }
    }

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, _delta: f32) {
        r.draw_text(&self.host_text, RENDER_WIDTH / 2 - 50, 100, 20, DEBUG_COLOR);

        r.draw_text(&self.join_text, RENDER_WIDTH / 2 - 50, 120, 20, DEBUG_COLOR);
    }

    pub fn message(&mut self, msg: &Message) {
        match msg {
            Message::Net(msg) => match msg {
                NetMessage::Hosted => {
                    self.actions.insert(Action::Hosted);
                }
                NetMessage::Connected => {
                    self.actions.insert(Action::Connected);
                }
                NetMessage::Disconnected => {
                    self.actions.insert(Action::Disconnected);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop_last() {
            match action {
                Action::Host => {
                    if self.hosting {
                        return;
                    }

                    self.hosting = true;

                    bus.send(NetRequestMessage::Host);
                }
                Action::Join(host) => {
                    if self.joining {
                        return;
                    }

                    self.joining = true;

                    bus.send(NetRequestMessage::Connect(host));
                }
                Action::Hosted => {
                    self.hosted = true;
                    self.actions.insert(Action::Join("127.0.0.1".to_owned()));
                }
                Action::Connected => {
                    bus.send(StateRequestMessage::Set(State::Lobby));
                }
                Action::Disconnected => {
                    self.joining = false;
                }
            }
        }
    }
}
