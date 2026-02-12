#[derive(Debug, Clone)]
pub struct EthernetIface {
    pub name: String,
    pub operstate: String,
    pub carrier: Option<bool>,
    pub mac: Option<String>,
    pub speed_mbps: Option<u32>,
    pub ipv4: Vec<String>,
    pub ipv6: Vec<String>,
    pub gateway_v4: Option<String>,
    pub dns: Vec<String>,
}

impl EthernetIface {
    pub fn is_active(&self) -> bool {
        self.carrier == Some(true) && !self.ipv4.is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct EthernetState {
    pub ifaces: Vec<EthernetIface>,
}

impl EthernetState {
    pub fn empty() -> Self {
        Self { ifaces: Vec::new() }
    }

    pub fn has_adapter(&self) -> bool {
        !self.ifaces.is_empty()
    }

    pub fn has_active(&self) -> bool {
        self.ifaces.iter().any(EthernetIface::is_active)
    }
}
