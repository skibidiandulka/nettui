use crate::{
    backend::traits::WifiBackend,
    domain::wifi::{WifiNetwork, WifiState},
};
use anyhow::Result;
use std::collections::HashSet;
use std::{fs, path::Path};
use tokio::process::Command;

pub struct IwdBackend;

impl IwdBackend {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self, iface: &str) -> Result<()> {
        run_iwctl(&["station", iface, "scan"]).await
    }

    pub async fn connect(&self, iface: &str, ssid: &str) -> Result<()> {
        run_iwctl(&["station", iface, "connect", ssid]).await
    }

    pub async fn disconnect(&self, iface: &str) -> Result<()> {
        run_iwctl(&["station", iface, "disconnect"]).await
    }
}

impl WifiBackend for IwdBackend {
    fn query_state(&self) -> Result<WifiState> {
        let ifaces = list_wifi_ifaces();
        if ifaces.is_empty() {
            return Ok(WifiState::empty());
        }

        let iface = ifaces[0].clone();
        let connected_ssid = parse_connected_network(&iface);
        let mut known_networks = parse_known_networks();
        let mut new_networks = parse_networks(&iface);

        if let Some(conn) = &connected_ssid {
            for n in &mut known_networks {
                if &n.ssid == conn {
                    n.connected = true;
                }
            }
            for n in &mut new_networks {
                if &n.ssid == conn {
                    n.connected = true;
                }
            }
        }

        // Keep "new networks" focused on unknown SSIDs for UX consistency with impala.
        let known_ssids: HashSet<&str> = known_networks.iter().map(|n| n.ssid.as_str()).collect();
        new_networks.retain(|n| !known_ssids.contains(n.ssid.as_str()));

        Ok(WifiState {
            ifaces,
            connected_ssid,
            known_networks,
            new_networks,
        })
    }
}

fn list_wifi_ifaces() -> Vec<String> {
    let mut out = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/net") else {
        return out;
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name == "lo" {
            continue;
        }

        let p = Path::new("/sys/class/net").join(&name);
        if p.join("wireless").is_dir() || p.join("phy80211").exists() {
            out.push(name);
        }
    }

    out.sort();
    out
}

fn parse_connected_network(iface: &str) -> Option<String> {
    let out = std::process::Command::new("iwctl")
        .arg("station")
        .arg(iface)
        .arg("show")
        .output()
        .ok()?;

    if !out.status.success() {
        return None;
    }

    let txt = String::from_utf8_lossy(&out.stdout);
    for line in txt.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("Connected network") {
            let ssid = rest.trim().trim_start_matches(':').trim();
            if !ssid.is_empty() {
                return Some(ssid.to_string());
            }
        }
    }

    None
}

fn parse_networks(iface: &str) -> Vec<WifiNetwork> {
    let out = std::process::Command::new("iwctl")
        .arg("station")
        .arg(iface)
        .arg("get-networks")
        .output();

    let Ok(out) = out else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }

    let txt = String::from_utf8_lossy(&out.stdout);
    let mut nets = Vec::new();

    for raw in txt.lines() {
        let line = raw.trim();
        if line.is_empty()
            || line.starts_with("Available networks")
            || line.starts_with("Network")
            || line.starts_with("----")
        {
            continue;
        }

        let connected = line.starts_with('>') || line.starts_with('*');
        let cleaned = line.trim_start_matches('>').trim_start_matches('*').trim();

        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let ssid = parts[0].to_string();
        let security = parts.get(1).copied().unwrap_or("-").to_string();
        let signal = parts.last().copied().unwrap_or("-").to_string();

        nets.push(WifiNetwork {
            ssid,
            security,
            signal,
            connected,
        });
    }

    nets
}

fn parse_known_networks() -> Vec<WifiNetwork> {
    let out = std::process::Command::new("iwctl")
        .arg("known-networks")
        .arg("list")
        .output();

    let Ok(out) = out else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }

    let txt = String::from_utf8_lossy(&out.stdout);
    let mut nets = Vec::new();

    for raw in txt.lines() {
        let line = raw.trim();
        if line.is_empty()
            || line.starts_with("Known networks")
            || line.starts_with("Name")
            || line.starts_with("---")
        {
            continue;
        }

        let cleaned = line.trim_start_matches('>').trim_start_matches('*').trim();
        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        let ssid = parts[0].to_string();
        let security = parts.get(1).copied().unwrap_or("-").to_string();
        nets.push(WifiNetwork {
            ssid,
            security,
            signal: "-".to_string(),
            connected: false,
        });
    }

    nets
}

async fn run_iwctl(args: &[&str]) -> Result<()> {
    let out = Command::new("iwctl").args(args).output().await?;
    if out.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    Err(std::io::Error::other(if stderr.is_empty() {
        "iwctl failed".to_string()
    } else {
        stderr
    })
    .into())
}
