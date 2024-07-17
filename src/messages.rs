use crate::{commands::Command, states::State};

#[derive(Debug)]
pub enum Message {
    State(StateMessage),
    Net(NetMessage),
}

#[derive(Debug)]
pub enum StateMessage {
    Request(StateRequestMessage),
    // not currently used, only signals that a state has been set
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
    Synchronize(u8, Vec<u8>, u32),
    Commands(u8, usize, Vec<Command>),
}

#[derive(Debug)]
pub enum NetRequestMessage {
    Synchronize,
    Commands(usize, Vec<Command>),
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
