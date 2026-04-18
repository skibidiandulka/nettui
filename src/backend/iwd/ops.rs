use super::{IwdBackend, WifiShareCredentials, helpers::*};
use crate::domain::wifi_enterprise::WifiEnterpriseProfile;
use anyhow::{Context, Result};
use iwdrs::{modes::Mode, session::Session};
use std::fs;
use tokio::process::Command;

impl IwdBackend {
    pub async fn scan(&self, preferred_iface: Option<&str>) -> Result<()> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Station)
            .context("no wifi station found")?;
        station_scan(&connection, &device.path).await
    }

    pub async fn disconnect(&self, preferred_iface: Option<&str>) -> Result<()> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Station)
            .context("no wifi station found")?;
        station_disconnect(&connection, &device.path).await
    }

    pub async fn connect(&self, preferred_iface: Option<&str>, ssid: &str) -> Result<()> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Station)
            .context("no wifi station found")?;
        let discovered = station_discovered_networks(&connection, &device.path).await?;

        for (network_path, _) in discovered {
            let name = match network_name(&connection, &network_path).await {
                Ok(value) => value,
                Err(_) => continue,
            };
            if name == ssid {
                network_connect(&connection, &network_path).await?;
                return Ok(());
            }
        }

        Err(std::io::Error::other(format!("network not found: {ssid}")).into())
    }

    pub async fn connect_hidden(&self, preferred_iface: Option<&str>, ssid: &str) -> Result<()> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Station)
            .context("no wifi station found")?;
        station_connect_hidden_network(&connection, &device.path, ssid).await
    }

    pub async fn connect_with_passphrase(
        &self,
        preferred_iface: Option<&str>,
        ssid: &str,
        passphrase: &str,
    ) -> Result<()> {
        let devices = load_devices().await?;
        let iface = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Station)
            .map(|device| device.iface.clone())
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

    pub async fn toggle_power(&self, preferred_iface: Option<&str>) -> Result<bool> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Any)
            .context("no wifi device found")?;
        let next_power = !device.powered;
        device_set_power(&connection, &device.path, next_power).await?;
        Ok(next_power)
    }

    pub async fn load_share_credentials(&self, ssid: &str) -> Result<WifiShareCredentials> {
        let encoded_name = iwd_network_name(ssid);
        let path = format!("/var/lib/iwd/{encoded_name}.psk");
        let raw = read_protected_file(&path).await?;
        let passphrase = raw
            .lines()
            .find_map(|line| line.strip_prefix("Passphrase="))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
            .context("known network does not expose a passphrase")?;

        Ok(WifiShareCredentials {
            ssid: ssid.to_string(),
            passphrase,
        })
    }

    pub async fn write_enterprise_profile(
        &self,
        ssid: &str,
        profile: &WifiEnterpriseProfile,
    ) -> Result<()> {
        let encoded_name = iwd_network_name(ssid);
        let path = std::path::Path::new("/var/lib/iwd").join(format!("{encoded_name}.8021x"));
        let contents = profile.to_iwd_profile()?;
        write_protected_file(&path, &contents, "failed to write iwd enterprise profile").await
    }

    pub async fn read_enterprise_profile(
        &self,
        ssid: &str,
    ) -> Result<Option<WifiEnterpriseProfile>> {
        let encoded_name = iwd_network_name(ssid);
        let path = std::path::Path::new("/var/lib/iwd").join(format!("{encoded_name}.8021x"));
        if !path.exists() {
            return Ok(None);
        }

        let raw = read_protected_file(path.to_string_lossy().as_ref()).await?;
        Ok(Some(WifiEnterpriseProfile::from_iwd_profile(ssid, &raw)?))
    }

    pub async fn start_access_point(
        &self,
        preferred_iface: Option<&str>,
        ssid: &str,
        passphrase: &str,
    ) -> Result<()> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device = select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::Any)
            .context("no wifi device found")?;
        device_set_mode(&connection, &device.path, Mode::Ap).await?;

        match access_point_start(&connection, &device.path, ssid, passphrase).await {
            Ok(()) => Ok(()),
            Err(err) if should_retry_access_point_with_profile(&err.to_string()) => {
                write_ap_profile(ssid, passphrase).await?;
                access_point_start_profile(&connection, &device.path, ssid).await?;
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    pub async fn stop_access_point(&self, preferred_iface: Option<&str>) -> Result<()> {
        let connection = system_bus_connection().await?;
        let objects = load_managed_objects(&connection).await?;
        let devices = device_snapshots_from_objects(&objects);
        let device =
            select_device_snapshot(&devices, preferred_iface, WifiInterfaceRole::AccessPoint)
                .context("no access point interface found")?;
        access_point_stop(&connection, &device.path).await?;
        device_set_mode(&connection, &device.path, Mode::Station).await?;
        Ok(())
    }

    pub fn network_configuration_enabled(&self) -> bool {
        let Ok(raw) = fs::read_to_string("/etc/iwd/main.conf") else {
            return false;
        };

        let mut in_general = false;
        for line in raw.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                in_general = line.eq_ignore_ascii_case("[General]");
                continue;
            }
            if in_general
                && let Some((key, value)) = line.split_once('=')
                && key
                    .trim()
                    .eq_ignore_ascii_case("EnableNetworkConfiguration")
            {
                return value.trim().eq_ignore_ascii_case("true");
            }
        }
        false
    }
}

fn should_retry_access_point_with_profile(err: &str) -> bool {
    let lower = err.to_lowercase();
    lower.contains("invalid arguments")
        || lower.contains("argument type is wrong")
        || lower.contains("operation failed")
        || lower.contains("not supported")
        || lower.contains("failed")
}
