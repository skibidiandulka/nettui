use super::{IwdBackend, helpers::*};
use crate::domain::wifi::{WifiDeviceInfo, WifiNetwork, WifiState};
use anyhow::{Context, Result};
use iwdrs::{modes::Mode, session::Session};

impl IwdBackend {
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
}
