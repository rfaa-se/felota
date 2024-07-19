use engine::Engine;

mod bus;
mod commands;
mod components;
mod constants;
mod engine;
mod entities;
mod forge;
mod logic;
mod logs;
mod messages;
mod net;
mod packets;
mod render;
mod states;
mod systems;

fn main() {
    Engine::new().run();
}
