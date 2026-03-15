pub(super) mod helpers;
mod ops;
mod state;

pub struct IwdBackend;

#[derive(Debug, Clone)]
pub struct WifiShareCredentials {
    pub ssid: String,
    pub passphrase: String,
}

impl IwdBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IwdBackend {
    fn default() -> Self {
        Self::new()
    }
}
