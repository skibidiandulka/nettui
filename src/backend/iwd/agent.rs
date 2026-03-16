use anyhow::{Context, Result};
use iwdrs::{
    agent::{Agent, AgentManager, CancellationReason},
    error::agent::Canceled,
    network::Network,
    session::Session,
};
use std::sync::{Arc, Mutex};
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
    oneshot,
};

#[derive(Debug, Clone)]
pub enum AuthPromptRequestKind {
    Passphrase,
    PrivateKeyPassphrase,
    UserPassword { user_name: Option<String> },
    UserNameAndPassphrase,
}

#[derive(Debug)]
pub enum AuthAgentEvent {
    Prompt(AuthPromptRequest),
    Cancel { reason: String },
    Released,
}

#[derive(Debug)]
pub struct AuthPromptRequest {
    pub ssid: String,
    pub kind: AuthPromptRequestKind,
    response_tx: oneshot::Sender<AuthPromptResponse>,
}

impl AuthPromptRequest {
    pub fn into_parts(
        self,
    ) -> (
        String,
        AuthPromptRequestKind,
        oneshot::Sender<AuthPromptResponse>,
    ) {
        (self.ssid, self.kind, self.response_tx)
    }
}

#[derive(Debug)]
pub enum AuthPromptResponse {
    Secret(String),
    UserNameAndPassphrase {
        user_name: String,
        passphrase: String,
    },
    Cancel,
}

#[derive(Clone)]
struct NettuiAuthAgent {
    events: UnboundedSender<AuthAgentEvent>,
    cancel_tx: Arc<Mutex<Option<oneshot::Sender<()>>>>,
}

impl NettuiAuthAgent {
    fn new(events: UnboundedSender<AuthAgentEvent>) -> Self {
        Self {
            events,
            cancel_tx: Arc::new(Mutex::new(None)),
        }
    }

    async fn request(
        &self,
        network: &Network,
        kind: AuthPromptRequestKind,
    ) -> Result<AuthPromptResponse, Canceled> {
        let ssid = network.name().await.map_err(|_| Canceled {})?;
        let (response_tx, response_rx) = oneshot::channel();
        let (cancel_tx, cancel_rx) = oneshot::channel();
        self.cancel_tx
            .lock()
            .expect("auth agent cancel mutex poisoned")
            .replace(cancel_tx);

        self.events
            .send(AuthAgentEvent::Prompt(AuthPromptRequest {
                ssid,
                kind,
                response_tx,
            }))
            .map_err(|_| Canceled {})?;

        let result = tokio::select! {
            response = response_rx => response.map_err(|_| Canceled {}),
            _ = cancel_rx => Err(Canceled {}),
        }?;

        self.cancel_tx
            .lock()
            .expect("auth agent cancel mutex poisoned")
            .take();
        Ok(result)
    }

    fn cancel_pending(&self) {
        if let Some(cancel_tx) = self
            .cancel_tx
            .lock()
            .expect("auth agent cancel mutex poisoned")
            .take()
        {
            let _ = cancel_tx.send(());
        }
    }
}

impl Agent for NettuiAuthAgent {
    async fn request_passphrase(&self, network: &Network) -> Result<String, Canceled> {
        match self
            .request(network, AuthPromptRequestKind::Passphrase)
            .await?
        {
            AuthPromptResponse::Secret(secret) => Ok(secret),
            _ => Err(Canceled {}),
        }
    }

    async fn request_private_key_passphrase(&self, network: &Network) -> Result<String, Canceled> {
        match self
            .request(network, AuthPromptRequestKind::PrivateKeyPassphrase)
            .await?
        {
            AuthPromptResponse::Secret(secret) => Ok(secret),
            _ => Err(Canceled {}),
        }
    }

    async fn request_user_name_and_passphrase(
        &self,
        network: &Network,
    ) -> Result<(String, String), Canceled> {
        match self
            .request(network, AuthPromptRequestKind::UserNameAndPassphrase)
            .await?
        {
            AuthPromptResponse::UserNameAndPassphrase {
                user_name,
                passphrase,
            } => Ok((user_name, passphrase)),
            _ => Err(Canceled {}),
        }
    }

    async fn request_user_password(
        &self,
        network: &Network,
        user_name: Option<&String>,
    ) -> Result<String, Canceled> {
        match self
            .request(
                network,
                AuthPromptRequestKind::UserPassword {
                    user_name: user_name.cloned(),
                },
            )
            .await?
        {
            AuthPromptResponse::Secret(secret) => Ok(secret),
            _ => Err(Canceled {}),
        }
    }

    fn cancel(&self, reason: CancellationReason) {
        self.cancel_pending();
        let _ = self.events.send(AuthAgentEvent::Cancel {
            reason: format!("{reason:?}"),
        });
    }

    fn release(&self) {
        self.cancel_pending();
        let _ = self.events.send(AuthAgentEvent::Released);
    }
}

pub async fn register_auth_agent() -> Result<(AgentManager, UnboundedReceiver<AuthAgentEvent>)> {
    let session = Session::new().await.context("cannot access iwd service")?;
    let (events_tx, events_rx) = unbounded_channel();
    let manager = session
        .register_agent(NettuiAuthAgent::new(events_tx))
        .await
        .context("failed to register iwd auth agent")?;
    Ok((manager, events_rx))
}
