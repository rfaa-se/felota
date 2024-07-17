use std::collections::VecDeque;

use raylib::prelude::*;

use crate::{
    bus::Bus,
    constants::{DEBUG_COLOR, RENDER_HEIGHT},
    messages::{Message, NetMessage, NetRequestMessage},
};

pub struct System {
    logs: VecDeque<String>,
    command_logs: VecDeque<String>,
}

impl System {
    pub fn new() -> Self {
        Self {
            logs: VecDeque::new(),
            command_logs: VecDeque::new(),
        }
    }

    pub fn update(&mut self, _h: &mut RaylibHandle, _bus: &mut Bus) {}

    pub fn draw(&mut self, r: &mut RaylibTextureMode<RaylibDrawHandle>, _delta: f32) {
        self.logs.iter().fold(
            RENDER_HEIGHT as i32 - self.logs.len() as i32 * 10 - 62,
            |y, log| {
                r.draw_text(log, 3, y, 10, DEBUG_COLOR);
                y + 10
            },
        );

        self.command_logs.iter().fold(
            RENDER_HEIGHT as i32 - self.command_logs.len() as i32 * 10 - 272,
            |y, log| {
                r.draw_text(log, 3, y, 10, DEBUG_COLOR);
                y + 10
            },
        );
    }

    pub fn message(&mut self, msg: &Message) {
        match msg {
            Message::Net(NetMessage::Request(NetRequestMessage::Commands(_, _)))
            | Message::Net(NetMessage::Commands(_, _, _)) => {
                // only save the latest 20 logs
                if self.command_logs.len() > 20 {
                    self.command_logs.pop_front();
                }
                self.command_logs
                    .push_back(format!("{:?}", msg).replace("(", "->").replace(")", ""));
            }
            _ => {
                // only save the latest 20 logs
                if self.logs.len() > 20 {
                    self.logs.pop_front();
                }

                self.logs
                    .push_back(format!("{:?}", msg).replace("(", "->").replace(")", ""));
            }
        }
    }
}
