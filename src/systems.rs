use crate::{engine, logs, net, states};

pub struct Systems {
    pub engine: engine::System,
    pub states: states::System,
    pub logs: logs::System,
    pub net: net::System,
}

impl Systems {
    pub fn new() -> Self {
        Self {
            engine: engine::System::new(),
            states: states::System::new(),
            logs: logs::System::new(),
            net: net::System::new(),
        }
    }
}
