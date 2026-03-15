use crate::domain::wifi::{WifiDeviceInfo, WifiNetwork, WifiState};
use anyhow::{Context, Result};
use iwdrs::modes::Mode;
use iwdrs::session::Session;
use std::{collections::HashMap, fs, path::Path};
use tokio::process::Command;

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

        let session = Session::new().await.context("cannot access iwd service")?;
        let devices = load_devices(&session).await?;
        let station_iface = devices
            .iter()
            .find(|device| device.mode == Mode::Station)
            .map(|device| device.iface.clone());
        let access_point_iface = devices
            .iter()
            .find(|device| device.mode == Mode::Ap)
            .map(|device| device.iface.clone());

        let station = session.stations().await?.pop();
        let access_point = session.access_points().await?.pop();
        let device = devices
            .iter()
            .find(|device| device.mode == Mode::Ap)
            .or_else(|| devices.iter().find(|device| device.mode == Mode::Station));
        let device_mode = device.map(|device| device.mode).unwrap_or(Mode::Station);

        let connected_ssid = if device_mode == Mode::Station {
            if let Some(station) = &station {
                if let Some(n) = station.connected_network().await? {
                    n.name().await.ok()
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let known_meta = load_known_meta(&session).await;
        let discovered = if device_mode == Mode::Station {
            if let Some(station) = &station {
                station.discovered_networks().await?
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

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
        if device_mode == Mode::Station
            && let Some(station) = &station
            && let Ok(hidden_list) = station.get_hidden_networks().await
        {
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

        let device_mode_label = match device_mode {
            Mode::Ap => "access point".to_string(),
            Mode::Station => "station".to_string(),
        };

        let powered = if let Some(device) = device {
            if device.powered {
                "On".to_string()
            } else {
                "Off".to_string()
            }
        } else {
            "-".to_string()
        };

        let mut frequency = "-".to_string();
        let mut security = "-".to_string();
        let mut access_point_ssid = None;
        let mut access_point_clients = Vec::new();
        let state = if device_mode == Mode::Ap {
            if let Some(ap) = &access_point {
                if let Ok(Some(name)) = ap.name().await {
                    access_point_ssid = Some(name);
                }
                if let Ok(Some(freq)) = ap.frequency().await {
                    frequency = format!("{:.2} GHz", freq as f32 / 1000.0);
                }
                if let Ok(Some(cipher)) = ap.group_cipher().await {
                    security = cipher;
                }
                if let Ok(mut diagnostics) = session.access_points_diagnostics().await
                    && let Some(diag) = diagnostics.pop()
                    && let Ok(entries) = diag.get().await
                {
                    access_point_clients = entries
                        .iter()
                        .filter_map(|entry| entry.get("Address"))
                        .map(|value| value.trim_matches('"').to_string())
                        .collect();
                }
                match ap.has_started().await {
                    Ok(true) => "started".to_string(),
                    Ok(false) => "idle".to_string(),
                    Err(_) => "-".to_string(),
                }
            } else {
                "idle".to_string()
            }
        } else {
            if let Ok(mut diagnostics) = session.stations_diagnostics().await
                && let Some(diag) = diagnostics.pop()
                && let Ok(d) = diag.get().await
            {
                frequency = format!("{:.2} GHz", d.frequency_mhz as f32 / 1000.0);
                security = d.security.to_string();
            }
            if let Some(station) = &station {
                station
                    .state()
                    .await
                    .map(|v| v.to_string())
                    .unwrap_or_else(|_| "-".to_string())
            } else {
                "-".to_string()
            }
        };

        let scanning = if device_mode == Mode::Ap {
            if let Some(ap) = &access_point {
                ap.is_scanning()
                    .await
                    .map(|v| {
                        if v {
                            "Yes".to_string()
                        } else {
                            "No".to_string()
                        }
                    })
                    .unwrap_or_else(|_| "-".to_string())
            } else {
                "-".to_string()
            }
        } else if let Some(station) = &station {
            station
                .is_scanning()
                .await
                .map(|v| {
                    if v {
                        "Yes".to_string()
                    } else {
                        "No".to_string()
                    }
                })
                .unwrap_or_else(|_| "-".to_string())
        } else {
            "-".to_string()
        };

        let fallback_iface = ifaces.first().cloned().unwrap_or_else(|| "-".to_string());

        Ok(WifiState {
            ifaces,
            station_iface,
            access_point_iface,
            connected_ssid,
            access_point_ssid,
            access_point_clients,
            known_networks,
            unavailable_known_networks,
            new_networks,
            hidden_networks,
            device: Some(WifiDeviceInfo {
                iface: device
                    .map(|device| device.iface.clone())
                    .unwrap_or(fallback_iface),
                mode: device_mode_label,
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

    pub async fn connect_with_passphrase(&self, ssid: &str, passphrase: &str) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let iface = load_devices(&session)
            .await?
            .into_iter()
            .find(|device| device.mode == Mode::Station)
            .map(|device| device.iface)
            .context("no wifi adapter found")?;

        let out = Command::new("iwctl")
            .arg("--passphrase")
            .arg(passphrase)
            .arg("station")
            .arg(&iface)
            .arg("connect")
            .arg(ssid)
            .output()
            .await
            .context("failed to run iwctl")?;

        if out.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let msg = if !stderr.is_empty() { stderr } else { stdout };
        Err(std::io::Error::other(if msg.is_empty() {
            "iwctl connect failed".to_string()
        } else {
            msg
        })
        .into())
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

    pub async fn start_access_point(&self, ssid: &str, passphrase: &str) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let devices = load_devices(&session).await?;
        let target_iface = devices
            .iter()
            .find(|device| device.mode == Mode::Station)
            .or_else(|| devices.first())
            .map(|device| device.iface.clone())
            .context("no wifi device found")?;
        let device = find_device_by_name(&session, &target_iface)
            .await?
            .context("no wifi device found")?;
        device.set_mode(Mode::Ap).await?;
        let session = Session::new().await.context("cannot access iwd service")?;
        let access_point = session
            .access_points()
            .await?
            .pop()
            .context("no access point interface found")?;
        access_point.start(ssid, passphrase).await?;
        Ok(())
    }

    pub async fn stop_access_point(&self) -> Result<()> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let access_point = session
            .access_points()
            .await?
            .pop()
            .context("no access point interface found")?;
        access_point.stop().await?;
        if let Some(ap_iface) = load_devices(&session)
            .await?
            .iter()
            .find(|device| device.mode == Mode::Ap)
            .map(|device| device.iface.clone())
            && let Some(device) = find_device_by_name(&session, &ap_iface).await?
        {
            device.set_mode(Mode::Station).await?;
        }
        Ok(())
    }
}

impl Default for IwdBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
struct KnownMeta {
    security: String,
    hidden: bool,
    autoconnect: bool,
}

#[derive(Debug, Clone)]
struct DeviceSnapshot {
    iface: String,
    mode: Mode,
    powered: bool,
}

async fn load_devices(session: &Session) -> Result<Vec<DeviceSnapshot>> {
    let mut snapshots = Vec::new();
    for device in session.devices().await? {
        let Ok(iface) = device.name().await else {
            continue;
        };
        let mode = device.get_mode().await.unwrap_or(Mode::Station);
        let powered = device.is_powered().await.unwrap_or(false);
        snapshots.push(DeviceSnapshot {
            iface,
            mode,
            powered,
        });
    }
    Ok(snapshots)
}

async fn find_device_by_name(
    session: &Session,
    iface: &str,
) -> Result<Option<iwdrs::device::Device>> {
    for device in session.devices().await? {
        if device.name().await.ok().as_deref() == Some(iface) {
            return Ok(Some(device));
        }
    }
    Ok(None)
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
