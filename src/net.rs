use crate::{
    bus::Bus,
    commands::Command,
    messages::{Message, NetMessage, NetRequestMessage},
};

use raylib::prelude::*;

pub struct System {
    actions: Vec<Action>,
    client_id: u8,
    client_ids: Vec<u8>,
    seed: u32,
}

enum Action {
    Synchronize,
    SendCommands(usize, Vec<Command>),
}

impl System {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            client_id: 0,
            client_ids: vec![0],
            seed: 0,
        }
    }

    pub fn update(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(h, bus);
    }

    pub fn message(&mut self, msg: &Message) {
        if let Message::Net(NetMessage::Request(req)) = msg {
            match req {
                NetRequestMessage::Synchronize => self.actions.push(Action::Synchronize),
                NetRequestMessage::Commands(tick, cmds) => {
                    self.actions.push(Action::SendCommands(*tick, cmds.clone()))
                }
            }
        }
    }

    fn action(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Synchronize => {
                    self.seed = h.get_random_value::<i32>(0..i32::MAX) as u32;
                    h.set_random_seed(self.seed);

                    bus.send(NetMessage::Synchronize(
                        self.client_id,
                        self.client_ids.clone(),
                        self.seed,
                    ));
                }
                Action::SendCommands(tick, cmds) => {
                    bus.send(NetMessage::Commands(self.client_id, tick, cmds));
                }
            }
        }
    }
}
