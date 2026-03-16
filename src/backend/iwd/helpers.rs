use anyhow::{Context, Result};
use iwdrs::{device::Device, modes::Mode, session::Session};
use std::{collections::HashMap, fs, path::Path};
use tokio::process::Command;

#[derive(Debug, Clone)]
pub(super) struct KnownMeta {
    pub security: String,
    pub hidden: bool,
    pub autoconnect: bool,
}

#[derive(Debug, Clone)]
pub(super) struct DeviceSnapshot {
    pub iface: String,
    pub mode: Mode,
    pub powered: bool,
}

pub(super) async fn load_devices(session: &Session) -> Result<Vec<DeviceSnapshot>> {
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

pub(super) async fn find_device_by_name(session: &Session, iface: &str) -> Result<Option<Device>> {
    for device in session.devices().await? {
        if device.name().await.ok().as_deref() == Some(iface) {
            return Ok(Some(device));
        }
    }
    Ok(None)
}

pub(super) async fn write_ap_profile(ssid: &str, passphrase: &str) -> Result<()> {
    let dir = Path::new("/var/lib/iwd/ap");
    let path = dir.join(format!("{ssid}.ap"));
    let profile = format!(
        "[General]\nChannel=1\n\n[Security]\nPassphrase={}\nPairwiseCiphers=CCMP-128\nGroupCipher=CCMP-128\n\n[IPv4]\n",
        passphrase
    );

    write_protected_file(&path, &profile, "failed to write iwd AP profile").await
}

pub(super) fn shell_escape_single_quotes(input: &str) -> String {
    input.replace('\'', "'\"'\"'")
}

pub(super) fn iwd_network_name(name: &str) -> String {
    if name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ')
    {
        name.to_string()
    } else {
        format!("={}", hex::encode(name))
    }
}

pub(super) async fn read_protected_file(path: &str) -> Result<String> {
    if let Ok(raw) = fs::read_to_string(path) {
        return Ok(raw);
    }

    let escaped_path = shell_escape_single_quotes(path);

    let pkexec = Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg(format!("cat '{escaped_path}'"))
        .output()
        .await;
    if let Ok(out) = pkexec
        && out.status.success()
    {
        return Ok(String::from_utf8_lossy(&out.stdout).to_string());
    }

    let sudo = Command::new("sudo")
        .arg("-n")
        .arg("sh")
        .arg("-c")
        .arg(format!("cat '{escaped_path}'"))
        .output()
        .await
        .context("failed to read iwd network profile")?;

    if sudo.status.success() {
        return Ok(String::from_utf8_lossy(&sudo.stdout).to_string());
    }

    let stderr = String::from_utf8_lossy(&sudo.stderr).trim().to_string();
    Err(std::io::Error::other(if stderr.is_empty() {
        "failed to read iwd network profile".to_string()
    } else {
        stderr
    })
    .into())
}

pub(super) async fn write_protected_file(
    path: &Path,
    contents: &str,
    error_context: &str,
) -> Result<()> {
    if let Some(parent) = path.parent()
        && fs::create_dir_all(parent).is_ok()
        && fs::write(path, contents).is_ok()
    {
        return Ok(());
    }

    let escaped_path = shell_escape_single_quotes(path.to_string_lossy().as_ref());
    let parent = path
        .parent()
        .map(|value| value.to_string_lossy().to_string());
    let mkdir = parent
        .as_deref()
        .map(shell_escape_single_quotes)
        .map(|escaped_parent| format!("mkdir -p '{escaped_parent}' && "))
        .unwrap_or_default();
    let shell_cmd =
        format!("{mkdir}cat > '{escaped_path}' <<'NETTUI_FILE'\n{contents}\nNETTUI_FILE");

    let pkexec = Command::new("pkexec")
        .arg("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .output()
        .await;
    if let Ok(out) = pkexec
        && out.status.success()
    {
        return Ok(());
    }

    let sudo = Command::new("sudo")
        .arg("-n")
        .arg("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .output()
        .await
        .context(error_context.to_string())?;
    if sudo.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&sudo.stderr).trim().to_string();
    Err(std::io::Error::other(if stderr.is_empty() {
        error_context.to_string()
    } else {
        stderr
    })
    .into())
}

pub(super) async fn load_known_meta(session: &Session) -> HashMap<String, KnownMeta> {
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

pub(super) fn percent_signal(signal_dbm: i16) -> String {
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

pub(super) fn list_wifi_ifaces() -> Vec<String> {
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
