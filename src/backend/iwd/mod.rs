mod agent;
pub(super) mod helpers;
mod ops;
mod state;

use agent::register_auth_agent;
pub use agent::{AuthAgentEvent, AuthPromptRequest, AuthPromptRequestKind, AuthPromptResponse};
use anyhow::Result;
use iwdrs::agent::AgentManager;
use tokio::sync::mpsc::UnboundedReceiver;

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

impl IwdBackend {
    pub async fn register_auth_agent(
        &self,
    ) -> Result<(AgentManager, UnboundedReceiver<AuthAgentEvent>)> {
        register_auth_agent().await
    }
}
