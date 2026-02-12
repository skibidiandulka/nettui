#[derive(Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub security: String,
    pub signal: String,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct WifiState {
    pub ifaces: Vec<String>,
    pub connected_ssid: Option<String>,
    pub known_networks: Vec<WifiNetwork>,
    pub new_networks: Vec<WifiNetwork>,
}

impl WifiState {
    pub fn empty() -> Self {
        Self {
            ifaces: Vec::new(),
            connected_ssid: None,
            known_networks: Vec::new(),
            new_networks: Vec::new(),
        }
    }

    pub fn has_adapter(&self) -> bool {
        !self.ifaces.is_empty()
    }

    pub fn is_active(&self) -> bool {
        self.connected_ssid.is_some()
    }
}
