use crate::commands::Command;

pub enum ClientPacket {
    Synchronize(u32, u32, Box<[u32]>),
    Commands(u32, u32, Box<[Command]>),
    Start,
}

pub enum ServerPacket {
    Commands(u32, Box<[Command]>),
}

const SYNCHRONIZE: u8 = 1;
const COMMANDS: u8 = 2;
const START: u8 = 3;

impl ClientPacket {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let Some((ptype, data)) = bytes.split_first() else {
            panic!("wtf pkt");
        };

        match *ptype {
            SYNCHRONIZE => {
                let (seed, data) = data.split_at(4);
                let seed = u32::from_be_bytes(seed.try_into().expect("wtf sync seed"));

                let (cid, data) = data.split_at(4);
                let cid = u32::from_be_bytes(cid.try_into().expect("wtf sync cid"));

                let mut cids = Vec::new();
                let mut read = 0;

                while read < data.len() {
                    let cid = data[read..].first_chunk::<4>().expect("wtf sync cids");
                    let cid = u32::from_be_bytes(*cid);

                    read += 4;

                    cids.push(cid);
                }

                ClientPacket::Synchronize(seed, cid, cids.into_boxed_slice())
            }
            COMMANDS => {
                let (cid, data) = data.split_at(4);
                let cid = u32::from_be_bytes(cid.try_into().expect("wtf cmds cid"));

                let (tick, data) = data.split_at(4);
                let tick = u32::from_be_bytes(tick.try_into().expect("wtf cmds tick"));

                let mut cmds = Vec::new();
                let mut read = 0;

                while read < data.len() {
                    let Some((len, data)) = data[read..].split_first() else {
                        panic!("wtf cmds");
                    };

                    let len = *len as usize;

                    read += 1;

                    cmds.push(Command::from_bytes(&data[..len]));

                    read += len;
                }

                ClientPacket::Commands(cid, tick, cmds.into_boxed_slice())
            }
            START => ClientPacket::Start,
            _ => panic!("wtf ptype {}", ptype),
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        match self {
            ClientPacket::Synchronize(seed, cid, cids) => {
                bytes.push(SYNCHRONIZE);
                bytes.extend_from_slice(&seed.to_be_bytes());
                bytes.extend_from_slice(&cid.to_be_bytes());

                for cid in cids.iter() {
                    bytes.extend_from_slice(&cid.to_be_bytes());
                }
            }
            ClientPacket::Commands(cid, tick, cmds) => {
                bytes.push(COMMANDS);
                bytes.extend_from_slice(&cid.to_be_bytes());
                bytes.extend_from_slice(&tick.to_be_bytes());

                for cmd in cmds.iter() {
                    bytes.extend_from_slice(&cmd.to_bytes());
                }
            }
            ClientPacket::Start => bytes.push(START),
        }

        bytes.into_boxed_slice()
    }
}

impl ServerPacket {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let Some((ptype, data)) = bytes.split_first() else {
            panic!("wtf pkt");
        };

        match *ptype {
            COMMANDS => {
                let (tick, data) = data.split_at(4);
                let tick = u32::from_be_bytes(tick.try_into().expect("wtf cmds tick"));

                let mut cmds = Vec::new();
                let mut read = 0;

                while read < data.len() {
                    let Some((len, data)) = data[read..].split_first() else {
                        panic!("wtf cmds");
                    };

                    let len = *len as usize;

                    read += 1;

                    cmds.push(Command::from_bytes(&data[..len]));

                    read += len;
                }

                ServerPacket::Commands(tick, cmds.into_boxed_slice())
            }
            _ => panic!("wtf ptype {}", ptype),
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        match self {
            ServerPacket::Commands(tick, cmds) => {
                bytes.push(COMMANDS);
                bytes.extend_from_slice(&tick.to_be_bytes());

                for cmd in cmds.iter() {
                    bytes.extend_from_slice(&cmd.to_bytes());
                }
            }
        }

        bytes.into_boxed_slice()
    }
}
