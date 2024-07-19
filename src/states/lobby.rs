use std::collections::BTreeSet;

use raylib::prelude::*;

use crate::{
    bus::Bus,
    constants::{DEBUG_COLOR, RENDER_WIDTH},
    messages::{Message, NetMessage, NetRequestMessage, StateRequestMessage},
};

use super::State;

pub struct Lobby {
    actions: BTreeSet<Action>,
    start_text: String,
    leave_text: String,
    client_id: u32,
    client_ids: Vec<u32>,
    seed: u32,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Action {
    Leave,
    Start,
    Play,
    Menu,
}

impl Lobby {
    pub fn new() -> Self {
        Self {
            actions: BTreeSet::new(),
            start_text: "[s]tart".to_owned(),
            leave_text: "[l]eave".to_owned(),
            client_id: 0,
            client_ids: Vec::new(),
            seed: 0,
        }
    }

    pub fn init(&mut self, bus: &mut Bus) {
        bus.send(NetRequestMessage::Synchronize);
    }

    pub fn exit(&mut self) {
        self.client_id = 0;
        self.client_ids.clear();
        self.seed = 0;
    }

    pub fn update(&mut self, _h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(bus);
    }

    pub fn input(&mut self, h: &mut RaylibHandle) {
        if h.is_key_pressed(KeyboardKey::KEY_L) {
            self.actions.insert(Action::Leave);
        }

        if h.is_key_pressed(KeyboardKey::KEY_S) {
            self.actions.insert(Action::Start);
        }
    }

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, _delta: f32) {
        r.draw_text(
            &self.leave_text,
            RENDER_WIDTH / 2 - 50,
            100,
            20,
            DEBUG_COLOR,
        );

        r.draw_text(
            &self.start_text,
            RENDER_WIDTH / 2 - 50,
            120,
            20,
            DEBUG_COLOR,
        );

        r.draw_text(
            &format!("cid {}", self.client_id),
            RENDER_WIDTH / 2 - 50,
            140,
            20,
            DEBUG_COLOR,
        );

        r.draw_text(
            &format!(
                "cids {}",
                self.client_ids
                    .iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            RENDER_WIDTH / 2 - 50,
            160,
            20,
            DEBUG_COLOR,
        );
    }

    pub fn message(&mut self, msg: &Message) {
        match msg {
            Message::Net(msg) => match msg {
                NetMessage::Synchronize(seed, cid, cids) => {
                    self.seed = *seed;
                    self.client_id = *cid;
                    self.client_ids = cids.to_vec();
                }
                NetMessage::Disconnected => {
                    self.actions.insert(Action::Menu);
                }
                NetMessage::Start => {
                    self.actions.insert(Action::Play);
                }
                _ => (),
            },
            _ => (),
        }
    }

    fn action(&mut self, bus: &mut Bus) {
        while let Some(action) = self.actions.pop_last() {
            match action {
                Action::Leave => {
                    bus.send(NetRequestMessage::Disconnect);
                }
                Action::Start => {
                    bus.send(NetRequestMessage::Start);
                }
                Action::Play => {
                    bus.send(StateRequestMessage::Set(State::Play));
                }
                Action::Menu => {
                    bus.send(StateRequestMessage::Set(State::Menu));
                }
            }
        }
    }
}
