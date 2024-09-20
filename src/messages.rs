use crate::{commands::Command, entities::EntityIndex, states::State};

#[derive(Debug)]
pub enum Message {
    State(StateMessage),
    Net(NetMessage),
    Engine(EngineMessage),
    Logic(LogicMessage),
}

#[derive(Debug)]
pub enum LogicMessage {
    EntityDead(usize, EntityIndex),
}

#[derive(Debug)]
pub enum EngineMessage {
    Request(EngineRequestMessage),
    // bool is currently not used anywhere
    #[allow(dead_code)]
    ToggleInterpolation(bool),
    ToggleDebug(bool),
    Synchronize(bool),
}

#[derive(Debug)]
pub enum EngineRequestMessage {
    ToggleInterpolation,
    ToggleDebug,
    Synchronize,
}

#[derive(Debug)]
pub enum StateMessage {
    Request(StateRequestMessage),
    // not currently used anywhere, only signals that a state has been set
    #[allow(dead_code)]
    Set(State),
}

#[derive(Debug)]
pub enum StateRequestMessage {
    Set(State),
}

#[derive(Debug)]
pub enum NetMessage {
    Request(NetRequestMessage),
    Synchronize(u32, u32, Box<[u32]>),
    Hosted,
    Connected,
    Disconnected,
    Start,
    Commands(u32, u32, Box<[Command]>),
    TogglePause(u32),
}

#[derive(Debug)]
pub enum NetRequestMessage {
    Synchronize,
    Host,
    Start,
    Connect(String),
    Disconnect,
    Commands(u32, Box<[Command]>),
    TogglePause,
}

impl Into<Message> for StateMessage {
    fn into(self) -> Message {
        Message::State(self)
    }
}

impl Into<Message> for StateRequestMessage {
    fn into(self) -> Message {
        Message::State(self.into())
    }
}

impl Into<StateMessage> for StateRequestMessage {
    fn into(self) -> StateMessage {
        StateMessage::Request(self)
    }
}

impl Into<Message> for NetMessage {
    fn into(self) -> Message {
        Message::Net(self)
    }
}

impl Into<Message> for NetRequestMessage {
    fn into(self) -> Message {
        Message::Net(self.into())
    }
}

impl Into<NetMessage> for NetRequestMessage {
    fn into(self) -> NetMessage {
        NetMessage::Request(self)
    }
}

impl Into<Message> for EngineMessage {
    fn into(self) -> Message {
        Message::Engine(self)
    }
}

impl Into<Message> for EngineRequestMessage {
    fn into(self) -> Message {
        Message::Engine(self.into())
    }
}

impl Into<EngineMessage> for EngineRequestMessage {
    fn into(self) -> EngineMessage {
        EngineMessage::Request(self)
    }
}

impl Into<Message> for LogicMessage {
    fn into(self) -> Message {
        Message::Logic(self)
    }
}
