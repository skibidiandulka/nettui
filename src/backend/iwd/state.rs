use super::{IwdBackend, helpers::*};
use crate::domain::wifi::{WifiDeviceInfo, WifiNetwork, WifiState};
use anyhow::{Context, Result};
use iwdrs::{modes::Mode, session::Session};

impl IwdBackend {
    pub async fn query_state(&self, preferred_iface: Option<&str>) -> Result<WifiState> {
        let ifaces = list_wifi_ifaces();
        if ifaces.is_empty() {
            return Ok(WifiState::empty());
        }

        let session = Session::new().await.context("cannot access iwd service")?;
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let station_device =
            select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Station);
        let access_point_device =
            select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::AccessPoint);
        let display_device = preferred_iface
            .and_then(|iface| devices.iter().find(|device| device.iface == iface))
            .or(access_point_device)
            .or(station_device)
            .or_else(|| select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Any));
        let station_iface = station_device.map(|device| device.iface.clone());
        let access_point_iface = access_point_device.map(|device| device.iface.clone());

        let connected_ssid = if let Some(device) = station_device {
            if let Some(path) = station_connected_network_path(&connection, &device.path).await? {
                network_name(&connection, &path).await.ok()
            } else {
                None
            }
        } else {
            None
        };

        let known_meta = load_known_meta(&session).await;
        let discovered = if let Some(device) = station_device {
            station_discovered_networks(&connection, &device.path).await?
        } else {
            Vec::new()
        };

        let mut known_networks = Vec::new();
        let mut new_networks = Vec::new();
        let mut available_names = std::collections::HashSet::new();

        for (network_path, signal_dbm) in discovered {
            let name = match network_name(&connection, &network_path).await {
                Ok(v) if !v.is_empty() => v,
                _ => continue,
            };
            let security = network_type_label(&connection, &network_path)
                .await
                .unwrap_or_else(|_| "-".to_string());
            let connected = network_connected(&connection, &network_path)
                .await
                .unwrap_or(false);
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
        if let Some(device) = station_device
            && let Ok(hidden_list) = station_hidden_networks(&connection, &device.path).await
        {
            for (address, signal_strength, network_type) in hidden_list {
                let security = network_type_display(&network_type);
                hidden_networks.push(WifiNetwork {
                    ssid: address,
                    security,
                    signal: percent_signal(signal_strength),
                    connected: false,
                    hidden: Some(true),
                    autoconnect: None,
                    available: false,
                });
            }
            hidden_networks.sort_by(|a, b| a.ssid.cmp(&b.ssid));
        }

        let mut access_point_ssid = None;
        let mut ap_clients = Vec::new();
        if let Some(device) = display_device.filter(|device| device.mode == Mode::Ap) {
            if let Ok(Some(name)) = access_point_name(&connection, &device.path).await {
                access_point_ssid = Some(name);
            }
            ap_clients = access_point_clients(&connection, &device.path)
                .await
                .unwrap_or_default();
        }

        let mut device_infos = Vec::new();
        for device in &devices {
            device_infos.push(build_device_info(&connection, device).await);
        }

        let device = display_device
            .and_then(|display| {
                device_infos
                    .iter()
                    .find(|device| device.iface == display.iface)
                    .cloned()
            })
            .or_else(|| device_infos.first().cloned());

        Ok(WifiState {
            ifaces,
            station_iface,
            access_point_iface,
            connected_ssid,
            access_point_ssid,
            access_point_clients: ap_clients,
            known_networks,
            unavailable_known_networks,
            new_networks,
            hidden_networks,
            devices: device_infos,
            device,
        })
    }
}

async fn build_device_info(
    connection: &zbus::Connection,
    device: &DeviceSnapshot,
) -> WifiDeviceInfo {
    let mode = match device.mode {
        Mode::Ap => "access point".to_string(),
        Mode::Station => "station".to_string(),
    };
    let powered = if device.powered {
        "On".to_string()
    } else {
        "Off".to_string()
    };

    let (state, scanning, frequency, security) =
        if device.mode == Mode::Ap && device.has_access_point {
            let state = device
                .access_point_started
                .map(|started| {
                    if started {
                        "started".to_string()
                    } else {
                        "idle".to_string()
                    }
                })
                .unwrap_or_else(|| "-".to_string());
            let scanning = access_point_is_scanning(connection, &device.path)
                .await
                .map(bool_label)
                .unwrap_or_else(|_| "-".to_string());
            let frequency = access_point_frequency(connection, &device.path)
                .await
                .ok()
                .flatten()
                .map(format_frequency)
                .unwrap_or_else(|| "-".to_string());
            let security = access_point_group_cipher(connection, &device.path)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| "-".to_string());
            (state, scanning, frequency, security)
        } else if device.has_station {
            let state = if let Some(state) = &device.station_state {
                state.clone()
            } else {
                station_state_label(connection, &device.path)
                    .await
                    .unwrap_or_else(|_| "-".to_string())
            };
            let scanning = if let Some(scanning) = device.station_scanning {
                bool_label(scanning)
            } else {
                station_is_scanning(connection, &device.path)
                    .await
                    .map(bool_label)
                    .unwrap_or_else(|_| "-".to_string())
            };
            let (frequency, security) = station_diagnostic_summary(connection, &device.path)
                .await
                .ok()
                .flatten()
                .unwrap_or_else(|| ("-".to_string(), "-".to_string()));
            (state, scanning, frequency, security)
        } else {
            (
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
                "-".to_string(),
            )
        };

    WifiDeviceInfo {
        iface: device.iface.clone(),
        mode,
        powered,
        state,
        scanning,
        frequency,
        security,
    }
}

fn bool_label(value: bool) -> String {
    if value {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

fn format_frequency(freq_mhz: u32) -> String {
    format!("{:.2} GHz", freq_mhz as f32 / 1000.0)
}
