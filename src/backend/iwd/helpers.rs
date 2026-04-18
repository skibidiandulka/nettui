use anyhow::{Context, Result};
use iwdrs::{modes::Mode, session::Session};
use std::{collections::HashMap, fs, path::Path};
use tokio::process::Command;
use zbus::{Connection, Proxy};
use zvariant::{OwnedObjectPath, OwnedValue};

const IWD_DESTINATION: &str = "net.connman.iwd";
const OBJECT_MANAGER_IFACE: &str = "org.freedesktop.DBus.ObjectManager";
const DEVICE_IFACE: &str = "net.connman.iwd.Device";
const STATION_IFACE: &str = "net.connman.iwd.Station";
const ACCESS_POINT_IFACE: &str = "net.connman.iwd.AccessPoint";
const STATION_DIAGNOSTIC_IFACE: &str = "net.connman.iwd.StationDiagnostic";
const ACCESS_POINT_DIAGNOSTIC_IFACE: &str = "net.connman.iwd.AccessPointDiagnostic";
const NETWORK_IFACE: &str = "net.connman.iwd.Network";

#[derive(Debug, Clone)]
pub(super) struct KnownMeta {
    pub security: String,
    pub hidden: bool,
    pub autoconnect: bool,
}

#[derive(Debug, Clone)]
pub(super) struct DeviceSnapshot {
    pub path: OwnedObjectPath,
    pub iface: String,
    pub mode: Mode,
    pub powered: bool,
    pub has_station: bool,
    pub has_access_point: bool,
    pub station_state: Option<String>,
    pub station_scanning: Option<bool>,
    pub access_point_started: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WifiInterfaceRole {
    Station,
    AccessPoint,
    Any,
}

type ManagedObjects = HashMap<OwnedObjectPath, HashMap<String, HashMap<String, OwnedValue>>>;

pub(super) async fn system_bus_connection() -> Result<Connection> {
    Connection::system()
        .await
        .context("cannot access iwd service")
}

pub(super) async fn load_managed_objects(connection: &Connection) -> Result<ManagedObjects> {
    let proxy = Proxy::new(connection, IWD_DESTINATION, "/", OBJECT_MANAGER_IFACE)
        .await
        .context("cannot access iwd service")?;
    proxy
        .call("GetManagedObjects", &())
        .await
        .context("cannot access iwd service")
}

pub(super) fn device_snapshots_from_objects(objects: &ManagedObjects) -> Vec<DeviceSnapshot> {
    let mut snapshots = Vec::new();

    for (path, interfaces) in objects {
        let Some(props) = interfaces.get(DEVICE_IFACE) else {
            continue;
        };

        let Some(iface) = props
            .get("Name")
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| String::try_from(value).ok())
        else {
            continue;
        };
        let mode = props
            .get("Mode")
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| String::try_from(value).ok())
            .map(|value| match value.as_str() {
                "ap" => Mode::Ap,
                _ => Mode::Station,
            })
            .unwrap_or(Mode::Station);
        let powered = props
            .get("Powered")
            .and_then(|value| value.try_clone().ok())
            .and_then(|value| bool::try_from(value).ok())
            .unwrap_or(false);

        snapshots.push(DeviceSnapshot {
            path: path.clone(),
            iface,
            mode,
            powered,
            has_station: interfaces.contains_key(STATION_IFACE),
            has_access_point: interfaces.contains_key(ACCESS_POINT_IFACE),
            station_state: interfaces
                .get(STATION_IFACE)
                .and_then(|props| props.get("State"))
                .and_then(|value| value.try_clone().ok())
                .and_then(|value| String::try_from(value).ok()),
            station_scanning: interfaces
                .get(STATION_IFACE)
                .and_then(|props| props.get("Scanning"))
                .and_then(|value| value.try_clone().ok())
                .and_then(|value| bool::try_from(value).ok()),
            access_point_started: interfaces
                .get(ACCESS_POINT_IFACE)
                .and_then(|props| props.get("Started"))
                .and_then(|value| value.try_clone().ok())
                .and_then(|value| bool::try_from(value).ok()),
        });
    }

    snapshots.sort_by(|a, b| a.iface.cmp(&b.iface));
    snapshots
}

pub(super) async fn load_devices() -> Result<Vec<DeviceSnapshot>> {
    let connection = system_bus_connection().await?;
    let objects = load_managed_objects(&connection).await?;
    Ok(device_snapshots_from_objects(&objects))
}

pub(super) fn select_device_snapshot<'a>(
    devices: &'a [DeviceSnapshot],
    preferred_iface: Option<&str>,
    role: WifiInterfaceRole,
) -> Option<&'a DeviceSnapshot> {
    if let Some(iface) = preferred_iface
        && let Some(device) = devices
            .iter()
            .find(|device| supports_role(device, role) && device.iface == iface)
    {
        return Some(device);
    }

    match role {
        WifiInterfaceRole::Station => devices
            .iter()
            .find(|device| supports_role(device, role) && station_is_live(device))
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| matches_role_mode(device, role) && device.powered)
            })
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| matches_role_mode(device, role))
            })
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| supports_role(device, role) && device.powered)
            })
            .or_else(|| devices.iter().find(|device| supports_role(device, role))),
        WifiInterfaceRole::AccessPoint => devices
            .iter()
            .find(|device| supports_role(device, role) && access_point_is_started(device))
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| matches_role_mode(device, role) && device.powered)
            })
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| matches_role_mode(device, role))
            })
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| supports_role(device, role) && device.powered)
            })
            .or_else(|| devices.iter().find(|device| supports_role(device, role))),
        WifiInterfaceRole::Any => devices
            .iter()
            .find(|device| access_point_is_started(device))
            .or_else(|| devices.iter().find(|device| station_is_live(device)))
            .or_else(|| {
                devices
                    .iter()
                    .find(|device| supports_role(device, role) && device.powered)
            })
            .or_else(|| devices.iter().find(|device| supports_role(device, role))),
    }
}

fn supports_role(device: &DeviceSnapshot, role: WifiInterfaceRole) -> bool {
    match role {
        WifiInterfaceRole::Station => device.has_station,
        WifiInterfaceRole::AccessPoint => device.has_access_point,
        WifiInterfaceRole::Any => device.has_station || device.has_access_point,
    }
}

fn matches_role_mode(device: &DeviceSnapshot, role: WifiInterfaceRole) -> bool {
    supports_role(device, role)
        && match role {
            WifiInterfaceRole::Station => device.mode == Mode::Station,
            WifiInterfaceRole::AccessPoint => device.mode == Mode::Ap,
            WifiInterfaceRole::Any => true,
        }
}

fn station_is_live(device: &DeviceSnapshot) -> bool {
    device.station_state.as_deref().is_some_and(|state| {
        state.eq_ignore_ascii_case("connected")
            || state.eq_ignore_ascii_case("connecting")
            || state.eq_ignore_ascii_case("roaming")
    }) || device.station_scanning.unwrap_or(false)
}

fn access_point_is_started(device: &DeviceSnapshot) -> bool {
    device.access_point_started.unwrap_or(false)
}

pub(super) async fn station_scan(connection: &Connection, path: &OwnedObjectPath) -> Result<()> {
    station_proxy(connection, path)
        .await?
        .call_method("Scan", &())
        .await?;
    Ok(())
}

pub(super) async fn station_disconnect(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<()> {
    station_proxy(connection, path)
        .await?
        .call_method("Disconnect", &())
        .await?;
    Ok(())
}

pub(super) async fn station_connect_hidden_network(
    connection: &Connection,
    path: &OwnedObjectPath,
    ssid: &str,
) -> Result<()> {
    station_proxy(connection, path)
        .await?
        .call_method("ConnectHiddenNetwork", &(ssid))
        .await?;
    Ok(())
}

pub(super) async fn station_is_scanning(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<bool> {
    Ok(station_proxy(connection, path)
        .await?
        .get_property("Scanning")
        .await?)
}

pub(super) async fn station_state_label(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<String> {
    Ok(station_proxy(connection, path)
        .await?
        .get_property("State")
        .await?)
}

pub(super) async fn station_connected_network_path(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Option<OwnedObjectPath>> {
    let proxy = station_proxy(connection, path).await?;
    let state: String = proxy.get_property("State").await?;
    if state.eq_ignore_ascii_case("connected") {
        return Ok(Some(proxy.get_property("ConnectedNetwork").await?));
    }
    Ok(None)
}

pub(super) async fn station_discovered_networks(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Vec<(OwnedObjectPath, i16)>> {
    let message = station_proxy(connection, path)
        .await?
        .call_method("GetOrderedNetworks", &())
        .await?;
    Ok(message.body().deserialize()?)
}

pub(super) async fn station_hidden_networks(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Vec<(String, i16, String)>> {
    let message = station_proxy(connection, path)
        .await?
        .call_method("GetHiddenAccessPoints", &())
        .await?;
    Ok(message.body().deserialize()?)
}

pub(super) async fn station_diagnostic_summary(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Option<(String, String)>> {
    let Ok(message) = diagnostic_proxy(connection, path, STATION_DIAGNOSTIC_IFACE)
        .await?
        .call_method("GetDiagnostics", &())
        .await
    else {
        return Ok(None);
    };

    let body: HashMap<String, OwnedValue> = message.body().deserialize()?;
    let frequency = body
        .get("Frequency")
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| u32::try_from(value).ok())
        .map(|mhz| format!("{:.2} GHz", mhz as f32 / 1000.0))
        .unwrap_or_else(|| "-".to_string());
    let security = body
        .get("Security")
        .and_then(|value| value.try_clone().ok())
        .and_then(|value| String::try_from(value).ok())
        .unwrap_or_else(|| "-".to_string());

    Ok(Some((frequency, security)))
}

pub(super) async fn access_point_start(
    connection: &Connection,
    path: &OwnedObjectPath,
    ssid: &str,
    passphrase: &str,
) -> Result<()> {
    access_point_proxy(connection, path)
        .await?
        .call_method("Start", &(ssid, passphrase))
        .await?;
    Ok(())
}

pub(super) async fn access_point_start_profile(
    connection: &Connection,
    path: &OwnedObjectPath,
    ssid: &str,
) -> Result<()> {
    access_point_proxy(connection, path)
        .await?
        .call_method("StartProfile", &(ssid))
        .await?;
    Ok(())
}

pub(super) async fn access_point_stop(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<()> {
    access_point_proxy(connection, path)
        .await?
        .call_method("Stop", &())
        .await?;
    Ok(())
}

pub(super) async fn access_point_is_scanning(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<bool> {
    Ok(access_point_proxy(connection, path)
        .await?
        .get_property("Scanning")
        .await?)
}

pub(super) async fn access_point_name(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Option<String>> {
    Ok(access_point_proxy(connection, path)
        .await?
        .get_property("Name")
        .await
        .ok())
}

pub(super) async fn access_point_frequency(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Option<u32>> {
    Ok(access_point_proxy(connection, path)
        .await?
        .get_property("Frequency")
        .await
        .ok())
}

pub(super) async fn access_point_group_cipher(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Option<String>> {
    Ok(access_point_proxy(connection, path)
        .await?
        .get_property("GroupCipher")
        .await
        .ok())
}

pub(super) async fn access_point_clients(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Vec<String>> {
    let Ok(message) = diagnostic_proxy(connection, path, ACCESS_POINT_DIAGNOSTIC_IFACE)
        .await?
        .call_method("GetDiagnostics", &())
        .await
    else {
        return Ok(Vec::new());
    };

    let body: Vec<HashMap<String, OwnedValue>> = message.body().deserialize()?;
    Ok(body
        .iter()
        .filter_map(|entry| entry.get("Address"))
        .filter_map(|value| value.try_clone().ok())
        .filter_map(|value| String::try_from(value).ok())
        .map(|value| value.trim_matches('"').to_string())
        .collect())
}

pub(super) async fn network_name(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<String> {
    Ok(network_proxy(connection, path)
        .await?
        .get_property("Name")
        .await?)
}

pub(super) async fn network_connected(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<bool> {
    Ok(network_proxy(connection, path)
        .await?
        .get_property("Connected")
        .await?)
}

pub(super) async fn network_type_label(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<String> {
    let value: String = network_proxy(connection, path)
        .await?
        .get_property("Type")
        .await?;
    Ok(network_type_display(&value))
}

pub(super) fn network_type_display(value: &str) -> String {
    match value {
        "open" => "Open".to_string(),
        "wep" => "Wep".to_string(),
        "psk" => "Psk".to_string(),
        "8021x" => "Eap".to_string(),
        _ => value.to_string(),
    }
}

pub(super) async fn network_connect(connection: &Connection, path: &OwnedObjectPath) -> Result<()> {
    network_proxy(connection, path)
        .await?
        .call_method("Connect", &())
        .await?;
    Ok(())
}

pub(super) async fn device_set_mode(
    connection: &Connection,
    path: &OwnedObjectPath,
    mode: Mode,
) -> Result<()> {
    device_proxy(connection, path)
        .await?
        .set_property("Mode", mode.to_string())
        .await?;
    Ok(())
}

pub(super) async fn device_set_power(
    connection: &Connection,
    path: &OwnedObjectPath,
    powered: bool,
) -> Result<()> {
    device_proxy(connection, path)
        .await?
        .set_property("Powered", powered)
        .await?;
    Ok(())
}

async fn device_proxy(connection: &Connection, path: &OwnedObjectPath) -> Result<Proxy<'static>> {
    proxy(connection, path, DEVICE_IFACE).await
}

async fn station_proxy(connection: &Connection, path: &OwnedObjectPath) -> Result<Proxy<'static>> {
    proxy(connection, path, STATION_IFACE).await
}

async fn access_point_proxy(
    connection: &Connection,
    path: &OwnedObjectPath,
) -> Result<Proxy<'static>> {
    proxy(connection, path, ACCESS_POINT_IFACE).await
}

async fn network_proxy(connection: &Connection, path: &OwnedObjectPath) -> Result<Proxy<'static>> {
    proxy(connection, path, NETWORK_IFACE).await
}

async fn diagnostic_proxy(
    connection: &Connection,
    path: &OwnedObjectPath,
    interface: &str,
) -> Result<Proxy<'static>> {
    proxy(connection, path, interface).await
}

async fn proxy(
    connection: &Connection,
    path: &OwnedObjectPath,
    interface: &str,
) -> Result<Proxy<'static>> {
    Proxy::new_owned(
        connection.clone(),
        IWD_DESTINATION,
        path.clone(),
        interface.to_string(),
    )
    .await
    .context("cannot access iwd service")
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

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(
        iface: &str,
        mode: Mode,
        powered: bool,
        has_station: bool,
        has_access_point: bool,
    ) -> DeviceSnapshot {
        DeviceSnapshot {
            path: OwnedObjectPath::try_from(format!("/net/connman/iwd/{iface}")).unwrap(),
            iface: iface.to_string(),
            mode,
            powered,
            has_station,
            has_access_point,
            station_state: None,
            station_scanning: None,
            access_point_started: None,
        }
    }

    #[test]
    fn preferred_iface_wins_for_station_selection() {
        let devices = vec![
            snapshot("wlan0", Mode::Station, true, true, false),
            snapshot("wlp2s0", Mode::Station, true, true, false),
        ];

        let selected =
            select_device_snapshot(&devices, Some("wlp2s0"), WifiInterfaceRole::Station).unwrap();
        assert_eq!(selected.iface, "wlp2s0");
    }

    #[test]
    fn powered_matching_mode_is_preferred() {
        let devices = vec![
            snapshot("wlan0", Mode::Station, false, true, false),
            snapshot("wlp2s0", Mode::Station, true, true, false),
        ];

        let selected = select_device_snapshot(&devices, None, WifiInterfaceRole::Station).unwrap();
        assert_eq!(selected.iface, "wlp2s0");
    }

    #[test]
    fn any_role_can_pick_access_point_device() {
        let devices = vec![
            snapshot("wlan0", Mode::Station, false, true, false),
            snapshot("wlp2s0", Mode::Ap, true, false, true),
        ];

        let selected = select_device_snapshot(&devices, None, WifiInterfaceRole::Any).unwrap();
        assert_eq!(selected.iface, "wlp2s0");
    }

    #[test]
    fn connected_station_is_preferred_over_idle_station() {
        let mut connected = snapshot("wlp2s0", Mode::Station, true, true, false);
        connected.station_state = Some("connected".to_string());
        let devices = vec![
            snapshot("wlan0", Mode::Station, true, true, false),
            connected,
        ];

        let selected = select_device_snapshot(&devices, None, WifiInterfaceRole::Station).unwrap();
        assert_eq!(selected.iface, "wlp2s0");
    }
}
