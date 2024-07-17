use std::collections::VecDeque;

use crate::{messages::Message, systems::Systems};

pub struct Bus {
    messages: VecDeque<Message>,
}

impl Bus {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
        }
    }

    pub fn update(&mut self, m: &mut Systems) {
        while let Some(msg) = self.messages.pop_front() {
            m.engine.message(&msg);
            m.net.message(&msg);
            m.logs.message(&msg);
            m.states.message(&msg);
        }
    }

    pub fn send<T: Into<Message>>(&mut self, msg: T) {
        self.messages.push_back(msg.into());
    }
}
