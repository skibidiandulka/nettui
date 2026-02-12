use crate::domain::ethernet::EthernetIface;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub program: String,
    pub args: Vec<String>,
    pub used_sudo: bool,
    pub status: i32,
    pub stdout: String,
    pub stderr: String,
}

pub trait EthernetBackend {
    fn list_ifaces(&self) -> Result<Vec<EthernetIface>>;
}
