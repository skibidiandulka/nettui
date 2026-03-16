use super::{IwdBackend, WifiShareCredentials, helpers::*};
use crate::domain::wifi_enterprise::WifiEnterpriseProfile;
use anyhow::{Context, Result};
use iwdrs::{modes::Mode, session::Session};
use std::fs;
use tokio::process::Command;

impl IwdBackend {
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

    pub async fn toggle_power(&self) -> Result<bool> {
        let session = Session::new().await.context("cannot access iwd service")?;
        let devices = load_devices(&session).await?;
        let target_iface = devices
            .iter()
            .find(|device| device.mode == Mode::Ap)
            .or_else(|| devices.iter().find(|device| device.mode == Mode::Station))
            .or_else(|| devices.first())
            .map(|device| device.iface.clone())
            .context("no wifi device found")?;
        let device = find_device_by_name(&session, &target_iface)
            .await?
            .context("no wifi device found")?;
        let next_power = !device.is_powered().await.unwrap_or(true);
        device.set_power(next_power).await?;
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

        match access_point.start(ssid, passphrase).await {
            Ok(()) => Ok(()),
            Err(err) if should_retry_access_point_with_profile(&err.to_string()) => {
                write_ap_profile(ssid, passphrase).await?;
                let session = Session::new().await.context("cannot access iwd service")?;
                let access_point = session
                    .access_points()
                    .await?
                    .pop()
                    .context("no access point interface found")?;
                access_point.start_profile(ssid).await?;
                Ok(())
            }
            Err(err) => Err(err.into()),
        }
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
