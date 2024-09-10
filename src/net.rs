use crate::{
    bus::Bus,
    commands::Command,
    messages::{Message, NetMessage, NetRequestMessage},
    packets::{ClientPacket, ServerPacket},
};

use raylib::prelude::*;
use redpine::{
    server::PeerHandle, Client, ClientEvent, SendMode, Server, ServerConfig, ServerEvent,
};

pub struct System {
    actions: Vec<Action>,
    seed: u32,
    server: Option<Server>,
    client: Option<Client>,
    clients: Vec<PeerHandle>,
    client_id: u32,
    client_ids: Vec<u32>,
}

enum Action {
    Synchronize,
    SendCommands(u32, Box<[Command]>),
    Create,
    Connect(String),
    Start,
    Disconnect,
    Shutdown,
}

const PORT: u16 = 1337;

impl System {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            seed: 0,
            server: None,
            client: None,
            clients: Vec::new(),
            client_id: 0,
            client_ids: Vec::new(),
        }
    }

    pub fn update(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        self.action(h, bus);
        self.client(h, bus);
        self.server();
    }

    pub fn message(&mut self, msg: &Message) {
        if let Message::Net(NetMessage::Request(req)) = msg {
            match req {
                NetRequestMessage::Synchronize => self.actions.push(Action::Synchronize),
                NetRequestMessage::Commands(tick, cmds) => {
                    self.actions.push(Action::SendCommands(*tick, cmds.clone()))
                }
                NetRequestMessage::Host => self.actions.push(Action::Create),
                NetRequestMessage::Connect(host) => {
                    self.actions.push(Action::Connect(host.clone()))
                }
                NetRequestMessage::Start => {
                    // can only start the game if we're hosting
                    if self.server.is_some() {
                        self.actions.push(Action::Start);
                    }
                }
                NetRequestMessage::Disconnect => self.actions.push(Action::Disconnect),
            }
        }
    }

    fn client(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        let Some(client) = self.client.as_mut() else {
            return;
        };

        while let Some(event) = client.poll_event() {
            match event {
                ClientEvent::Connect => {
                    bus.send(NetMessage::Connected);
                }
                ClientEvent::Disconnect => {
                    bus.send(NetMessage::Disconnected);
                }
                ClientEvent::Receive(data) => match ClientPacket::from_bytes(&data) {
                    ClientPacket::Synchronize(seed, cid, cids) => {
                        self.seed = seed;
                        self.client_id = cid;
                        self.client_ids = cids.to_vec();

                        h.set_random_seed(self.seed);

                        bus.send(NetMessage::Synchronize(seed, cid, cids));
                    }
                    ClientPacket::Commands(cid, tick, cmds) => {
                        bus.send(NetMessage::Commands(cid, tick, cmds.clone()));
                    }
                    ClientPacket::Start => {
                        bus.send(NetMessage::Start);
                    }
                },
                ClientEvent::Error(_) => {
                    bus.send(NetMessage::Disconnected);
                }
            }
        }
    }

    fn server(&mut self) {
        let Some(server) = self.server.as_mut() else {
            return;
        };

        while let Some(event) = server.poll_event() {
            match event {
                ServerEvent::Connect(peer) => {
                    let cid = peer.id();

                    self.clients.push(peer);
                    self.client_ids.push(cid);

                    // send a sync to all clients
                    for client in self.clients.iter_mut() {
                        client.send(
                            ClientPacket::Synchronize(
                                self.seed,
                                client.id(),
                                self.client_ids.clone().into_boxed_slice(),
                            )
                            .to_bytes(),
                            SendMode::Reliable,
                        );
                    }
                }
                ServerEvent::Disconnect(peer) => {
                    let cid = peer.id();

                    self.clients.retain(|x| x.id() != cid);
                    self.client_ids.retain(|x| *x != cid);

                    // we've been disconnected from our own server, let's kill it
                    if cid == self.client_id {
                        self.actions.push(Action::Shutdown);
                    } else {
                        // send a sync to all clients
                        for client in self.clients.iter_mut() {
                            client.send(
                                ClientPacket::Synchronize(
                                    self.seed,
                                    client.id(),
                                    self.client_ids.clone().into_boxed_slice(),
                                )
                                .to_bytes(),
                                SendMode::Reliable,
                            );
                        }
                    }
                }
                ServerEvent::Receive(peer, data) => match ServerPacket::from_bytes(&data) {
                    ServerPacket::Commands(tick, cmds) => {
                        let cid = peer.id();
                        for client in self.clients.iter_mut() {
                            client.send(
                                ClientPacket::Commands(cid, tick, cmds.clone()).to_bytes(),
                                SendMode::Reliable,
                            );
                        }
                    }
                },
                ServerEvent::Error(_peer, error) => match error {
                    redpine::ErrorKind::Timeout => todo!(),
                    redpine::ErrorKind::Capacity => todo!(),
                    redpine::ErrorKind::Parameter => todo!(),
                },
            }
        }
    }

    fn action(&mut self, h: &mut RaylibHandle, bus: &mut Bus) {
        while let Some(action) = self.actions.pop() {
            match action {
                Action::Synchronize => {
                    bus.send(NetMessage::Synchronize(
                        self.seed,
                        self.client_id,
                        self.client_ids.clone().into_boxed_slice(),
                    ));
                }
                Action::SendCommands(tick, cmds) => {
                    if let Some(client) = self.client.as_mut() {
                        client.send(
                            ServerPacket::Commands(tick, cmds).to_bytes(),
                            SendMode::Reliable,
                        );
                    }
                }
                Action::Create => {
                    if let Ok(server) = Server::bind_with_config(
                        ("127.0.0.1", PORT),
                        ServerConfig {
                            peer_count_max: 4,
                            ..Default::default()
                        },
                    ) {
                        self.server = Some(server);
                        self.seed = h.get_random_value::<i32>(0..i32::MAX) as u32;

                        h.set_random_seed(self.seed);

                        bus.send(NetMessage::Hosted);
                    }
                }
                Action::Connect(host) => {
                    if let Ok(client) = Client::connect((host, PORT)) {
                        self.client = Some(client);
                    }
                }
                Action::Start => {
                    for client in self.clients.iter_mut() {
                        client.send(ClientPacket::Start.to_bytes(), SendMode::Reliable);
                    }
                }
                Action::Disconnect => {
                    if let Some(client) = self.client.take().as_mut() {
                        client.disconnect();

                        self.client_id = 0;
                        self.client_ids.clear();

                        bus.send(NetMessage::Disconnected);
                    }
                }
                Action::Shutdown => {
                    if let Some(_) = self.server.take() {
                        for client in self.clients.iter_mut() {
                            client.disconnect();
                        }

                        self.clients.clear();
                    }
                }
            }
        }
    }
}
