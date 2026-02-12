use crate::domain::wifi::{WifiDeviceInfo, WifiNetwork, WifiState};
use anyhow::{Context, Result};
use iwdrs::session::Session;
use std::{collections::HashMap, fs, path::Path};

pub struct IwdBackend;

impl IwdBackend {
    pub fn new() -> Self {
        Self
    }

    pub async fn query_state(&self) -> Result<WifiState> {
        let ifaces = list_wifi_ifaces();
        if ifaces.is_empty() {
            return Ok(WifiState::empty());
        }
        let iface = ifaces[0].clone();

        let session = Session::new().await.context("cannot access iwd service")?;
        let station = session
            .stations()
            .await?
            .pop()
            .context("no wifi station found")?;

        let connected_ssid = if let Some(n) = station.connected_network().await? {
            n.name().await.ok()
        } else {
            None
        };

        let known_meta = load_known_meta(&session).await;
        let discovered = station.discovered_networks().await?;

        let mut known_networks = Vec::new();
        let mut new_networks = Vec::new();
        let mut available_names = std::collections::HashSet::new();

        for (network, signal_dbm) in discovered {
            let name = match network.name().await {
                Ok(v) if !v.is_empty() => v,
                _ => continue,
            };
            let security = network
                .network_type()
                .await
                .map(|v| v.to_string())
                .unwrap_or_else(|_| "-".to_string());
            let connected = connected_ssid.as_deref() == Some(name.as_str());
            let signal = percent_signal(signal_dbm);

            if let Some(meta) = known_meta.get(&name) {
                available_names.insert(name.clone());
                known_networks.push(WifiNetwork {
                    ssid: name,
                    security,
                    signal,
                    connected,
                    hidden: Some(meta.hidden),
                    autoconnect: Some(meta.autoconnect),
                    available: true,
                });
            } else {
                new_networks.push(WifiNetwork {
                    ssid: name,
                    security,
                    signal,
                    connected,
                    hidden: None,
                    autoconnect: None,
                    available: true,
                });
            }
        }

        known_networks.sort_by(|a, b| a.ssid.cmp(&b.ssid));
        new_networks.sort_by(|a, b| a.ssid.cmp(&b.ssid));

        let mut unavailable_known_networks = Vec::new();
        for (name, meta) in &known_meta {
            if available_names.contains(name) {
                continue;
            }
            unavailable_known_networks.push(WifiNetwork {
                ssid: name.clone(),
                security: meta.security.clone(),
                signal: "-".to_string(),
                connected: false,
                hidden: Some(meta.hidden),
                autoconnect: Some(meta.autoconnect),
                available: false,
            });
        }
        unavailable_known_networks.sort_by(|a, b| a.ssid.cmp(&b.ssid));

        let mut hidden_networks = Vec::new();
        if let Ok(hidden_list) = station.get_hidden_networks().await {
            for net in hidden_list {
                let security = net
                    .network_type
                    .to_string()
                    .split("::")
                    .last()
                    .unwrap_or("-")
                    .to_string();
                hidden_networks.push(WifiNetwork {
                    ssid: net.address,
                    security,
                    signal: percent_signal(net.signal_strength),
                    connected: false,
                    hidden: Some(true),
                    autoconnect: None,
                    available: false,
                });
            }
            hidden_networks.sort_by(|a, b| a.ssid.cmp(&b.ssid));
        }

        let state = station
            .state()
            .await
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "-".to_string());
        let scanning = station
            .is_scanning()
            .await
            .map(|v| {
                if v {
                    "Yes".to_string()
                } else {
                    "No".to_string()
                }
            })
            .unwrap_or_else(|_| "-".to_string());

        let powered = match session.devices().await {
            Ok(mut devices) => {
                if let Some(device) = devices.pop() {
                    match device.is_powered().await {
                        Ok(true) => "On".to_string(),
                        Ok(false) => "Off".to_string(),
                        Err(_) => "-".to_string(),
                    }
                } else {
                    "-".to_string()
                }
            }
            Err(_) => "-".to_string(),
        };

        let mut frequency = "-".to_string();
        let mut security = "-".to_string();
        if let Ok(mut diagnostics) = session.stations_diagnostics().await
            && let Some(diag) = diagnostics.pop()
            && let Ok(d) = diag.get().await
        {
            frequency = format!("{:.2} GHz", d.frequency_mhz as f32 / 1000.0);
            security = d.security.to_string();
        }

        Ok(WifiState {
            ifaces,
            connected_ssid,
            known_networks,
            unavailable_known_networks,
            new_networks,
            hidden_networks,
            device: Some(WifiDeviceInfo {
                iface,
                mode: "station".to_string(),
                powered,
                state,
                scanning,
                frequency,
                security,
            }),
        })
    }

    pub async fn scan(&self) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let station = session
            .stations()
            .await?
            .pop()
            .context("no wifi station found")?;
        station.scan().await?;
        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let station = session
            .stations()
            .await?
            .pop()
            .context("no wifi station found")?;
        station.disconnect().await?;
        Ok(())
    }

    pub async fn connect(&self, ssid: &str) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let station = session
            .stations()
            .await?
            .pop()
            .context("no wifi station found")?;
        let discovered = station.discovered_networks().await?;

        for (network, _) in discovered {
            let name = match network.name().await {
                Ok(v) => v,
                Err(_) => continue,
            };
            if name == ssid {
                network.connect().await?;
                return Ok(());
            }
        }

        Err(std::io::Error::other(format!("network not found: {ssid}")).into())
    }

    pub async fn connect_hidden(&self, ssid: &str) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let station = session
            .stations()
            .await?
            .pop()
            .context("no wifi station found")?;
        station.connect_hidden_network(ssid.to_string()).await?;
        Ok(())
    }

    pub async fn forget_known(&self, ssid: &str) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let known = session.known_networks().await?;
        for network in known {
            let name = network.name().await.unwrap_or_default();
            if name == ssid {
                network.forget().await?;
                return Ok(());
            }
        }
        Err(std::io::Error::other(format!("known network not found: {ssid}")).into())
    }

    pub async fn toggle_autoconnect(&self, ssid: &str) -> Result<bool> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let known = session.known_networks().await?;
        for network in known {
            let name = network.name().await.unwrap_or_default();
            if name == ssid {
                let current = network.get_autoconnect().await.unwrap_or(false);
                let next = !current;
                network.set_autoconnect(next).await?;
                return Ok(next);
            }
        }
        Err(std::io::Error::other(format!("known network not found: {ssid}")).into())
    }
}

#[derive(Debug, Clone)]
struct KnownMeta {
    security: String,
    hidden: bool,
    autoconnect: bool,
}

async fn load_known_meta(session: &Session) -> HashMap<String, KnownMeta> {
    let mut map = HashMap::new();
    let Ok(known) = session.known_networks().await else {
        return map;
    };

    for network in known {
        let name = network.name().await.unwrap_or_default();
        if name.is_empty() {
            continue;
        }
        let security = network
            .network_type()
            .await
            .map(|v| v.to_string())
            .unwrap_or_else(|_| "-".to_string());
        let hidden = network.hidden().await.unwrap_or(false);
        let autoconnect = network.get_autoconnect().await.unwrap_or(false);
        map.insert(
            name,
            KnownMeta {
                security,
                hidden,
                autoconnect,
            },
        );
    }

    map
}

fn percent_signal(signal_dbm: i16) -> String {
    let signal = if signal_dbm / 100 >= -50 {
        100
    } else {
        2 * (100 + signal_dbm / 100)
    };

    match signal {
        n if n >= 75 => format!("{signal:3}% 󰤨"),
        n if (50..75).contains(&n) => format!("{signal:3}% 󰤥"),
        n if (25..50).contains(&n) => format!("{signal:3}% 󰤢"),
        _ => format!("{signal:3}% 󰤟"),
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
