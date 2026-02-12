use crate::{
    backend::{
        iwd::IwdBackend,
        networkd::NetworkdBackend,
        traits::{EthernetBackend, WifiBackend},
    },
    domain::{
        common::{ActiveTab, StartupTabPolicy, Toast, ToastKind},
        ethernet::{EthernetIface, EthernetState},
        wifi::{WifiNetwork, WifiState},
    },
};
use anyhow::Result;
use ratatui::widgets::TableState;
use std::time::{Duration, Instant};
use tokio::process::Command;

#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub startup_policy: StartupTabPolicy,
    pub tick_ms: u64,
    pub esc_quit: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            startup_policy: StartupTabPolicy::PreferActive,
            tick_ms: 250,
            esc_quit: true,
        }
    }
}

pub struct App {
    pub running: bool,
    pub config: AppConfig,
    pub active_tab: ActiveTab,

    pub wifi: WifiState,
    pub wifi_state: TableState,

    pub ethernet: EthernetState,
    pub ethernet_state: TableState,

    pub last_error: Option<String>,
    pub last_action: Option<String>,
    pub toast: Option<Toast>,

    wifi_backend: IwdBackend,
    eth_backend: NetworkdBackend,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let wifi_backend = IwdBackend::new();
        let eth_backend = NetworkdBackend::new();

        let wifi = wifi_backend
            .query_state()
            .unwrap_or_else(|_| WifiState::empty());
        let ethernet = EthernetState {
            ifaces: eth_backend.list_ifaces().unwrap_or_default(),
        };

        let active_tab = determine_start_tab(config.startup_policy, &wifi, &ethernet);

        let mut wifi_state = TableState::default();
        if wifi.networks.is_empty() {
            wifi_state.select(None);
        } else {
            wifi_state.select(Some(0));
        }

        let mut ethernet_state = TableState::default();
        if ethernet.ifaces.is_empty() {
            ethernet_state.select(None);
        } else {
            ethernet_state.select(Some(0));
        }

        Ok(Self {
            running: true,
            config,
            active_tab,
            wifi,
            wifi_state,
            ethernet,
            ethernet_state,
            last_error: None,
            last_action: None,
            toast: None,
            wifi_backend,
            eth_backend,
        })
    }

    pub async fn tick(&mut self) -> Result<()> {
        if let Some(t) = &self.toast
            && Instant::now() >= t.until
        {
            self.toast = None;
        }

        self.refresh_all().await;
        Ok(())
    }

    async fn refresh_all(&mut self) {
        if let Ok(wifi) = self.wifi_backend.query_state() {
            let selected = self.wifi_state.selected();
            self.wifi = wifi;
            if self.wifi.networks.is_empty() {
                self.wifi_state.select(None);
            } else if let Some(i) = selected {
                self.wifi_state
                    .select(Some(i.min(self.wifi.networks.len().saturating_sub(1))));
            } else {
                self.wifi_state.select(Some(0));
            }
        }

        if let Ok(ifaces) = self.eth_backend.list_ifaces() {
            let selected = self.ethernet_state.selected();
            self.ethernet = EthernetState { ifaces };
            if self.ethernet.ifaces.is_empty() {
                self.ethernet_state.select(None);
            } else if let Some(i) = selected {
                self.ethernet_state
                    .select(Some(i.min(self.ethernet.ifaces.len().saturating_sub(1))));
            } else {
                self.ethernet_state.select(Some(0));
            }
        }

        self.last_error = None;
    }

    pub async fn refresh_current(&mut self) {
        self.refresh_all().await;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn switch_tab_next(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Wifi => ActiveTab::Ethernet,
            ActiveTab::Ethernet => ActiveTab::Wifi,
        };
    }

    pub fn switch_tab_prev(&mut self) {
        self.switch_tab_next();
    }

    pub fn select_next(&mut self) {
        match self.active_tab {
            ActiveTab::Wifi => {
                if self.wifi.networks.is_empty() {
                    self.wifi_state.select(None);
                    return;
                }
                let i = match self.wifi_state.selected() {
                    Some(i) => (i + 1).min(self.wifi.networks.len() - 1),
                    None => 0,
                };
                self.wifi_state.select(Some(i));
            }
            ActiveTab::Ethernet => {
                if self.ethernet.ifaces.is_empty() {
                    self.ethernet_state.select(None);
                    return;
                }
                let i = match self.ethernet_state.selected() {
                    Some(i) => (i + 1).min(self.ethernet.ifaces.len() - 1),
                    None => 0,
                };
                self.ethernet_state.select(Some(i));
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.active_tab {
            ActiveTab::Wifi => {
                if self.wifi.networks.is_empty() {
                    self.wifi_state.select(None);
                    return;
                }
                let i = match self.wifi_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                self.wifi_state.select(Some(i));
            }
            ActiveTab::Ethernet => {
                if self.ethernet.ifaces.is_empty() {
                    self.ethernet_state.select(None);
                    return;
                }
                let i = match self.ethernet_state.selected() {
                    Some(i) => i.saturating_sub(1),
                    None => 0,
                };
                self.ethernet_state.select(Some(i));
            }
        }
    }

    pub fn selected_wifi_network(&self) -> Option<&WifiNetwork> {
        self.wifi_state
            .selected()
            .and_then(|i| self.wifi.networks.get(i))
    }

    pub fn selected_eth_iface(&self) -> Option<&EthernetIface> {
        self.ethernet_state
            .selected()
            .and_then(|i| self.ethernet.ifaces.get(i))
    }

    pub fn set_toast(&mut self, kind: ToastKind, msg: impl Into<String>) {
        self.toast = Some(Toast {
            kind,
            msg: msg.into(),
            until: Instant::now() + Duration::from_millis(3000),
        });
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub async fn notify(&self, title: &str, body: &str) {
        let _ = Command::new("notify-send")
            .arg(title)
            .arg(body)
            .arg("-t")
            .arg("2000")
            .output()
            .await;
    }

    pub async fn wifi_scan(&mut self) -> Result<()> {
        let iface = self
            .wifi
            .ifaces
            .first()
            .cloned()
            .ok_or_else(|| std::io::Error::other("no wifi adapter found"))?;
        self.wifi_backend.scan(&iface).await?;
        self.last_action = Some(format!("Wi-Fi scan requested on {iface}"));
        self.set_toast(ToastKind::Info, format!("Wi-Fi scan requested on {iface}"));
        self.notify("Wi-Fi", &format!("Scan requested on {iface}"))
            .await;
        Ok(())
    }

    pub async fn wifi_connect_or_disconnect(&mut self) -> Result<()> {
        let iface = self
            .wifi
            .ifaces
            .first()
            .cloned()
            .ok_or_else(|| std::io::Error::other("no wifi adapter found"))?;

        let Some(net) = self.selected_wifi_network().cloned() else {
            return Err(std::io::Error::other("no network selected").into());
        };

        if net.connected {
            self.wifi_backend.disconnect(&iface).await?;
            self.last_action = Some(format!("Disconnected Wi-Fi on {iface}"));
            self.set_toast(
                ToastKind::Success,
                format!("Disconnected from {}", net.ssid),
            );
            self.notify("Wi-Fi", &format!("Disconnected from {}", net.ssid))
                .await;
        } else {
            self.wifi_backend.connect(&iface, &net.ssid).await?;
            self.last_action = Some(format!("Connect requested to {}", net.ssid));
            self.set_toast(
                ToastKind::Success,
                format!("Connect requested to {}", net.ssid),
            );
            self.notify("Wi-Fi", &format!("Connect requested to {}", net.ssid))
                .await;
        }

        Ok(())
    }

    pub async fn ethernet_renew_dhcp(&mut self) -> Result<()> {
        let iface = self
            .selected_eth_iface()
            .map(|i| i.name.clone())
            .ok_or_else(|| std::io::Error::other("no ethernet interface selected"))?;

        let before = snapshot_eth(self.selected_eth_iface());
        let out = self.eth_backend.renew_dhcp(&iface).await?;
        self.refresh_all().await;
        let after = snapshot_eth(self.selected_eth_iface());

        let mut msg = format!("{iface}: DHCP renew requested");
        if out.used_sudo {
            msg.push_str(" (sudo)");
        }
        if !out.stdout.is_empty() {
            msg.push_str(&format!("\nstdout: {}", out.stdout));
        }
        if !out.stderr.is_empty() {
            msg.push_str(&format!("\nstderr: {}", out.stderr));
        }
        if before == after {
            msg.push_str("\nNo visible change detected.");
        }
        msg.push_str(&format!("\nBefore: {before}\nAfter:  {after}"));

        self.last_action = Some(format!("Renewed DHCP on {iface}"));
        self.set_toast(ToastKind::Success, msg);
        self.notify("Ethernet", &format!("DHCP renew requested on {iface}"))
            .await;
        Ok(())
    }
}

pub fn determine_start_tab(
    policy: StartupTabPolicy,
    wifi: &WifiState,
    ethernet: &EthernetState,
) -> ActiveTab {
    match policy {
        StartupTabPolicy::ForceWifi => ActiveTab::Wifi,
        StartupTabPolicy::ForceEthernet => ActiveTab::Ethernet,
        StartupTabPolicy::PreferActive => {
            if ethernet.has_active() {
                ActiveTab::Ethernet
            } else if wifi.is_active() {
                ActiveTab::Wifi
            } else if wifi.has_adapter() {
                ActiveTab::Wifi
            } else {
                ActiveTab::Ethernet
            }
        }
    }
}

fn snapshot_eth(iface: Option<&EthernetIface>) -> String {
    let Some(i) = iface else {
        return "no interface selected".to_string();
    };

    let carrier = i.carrier.map(|c| if c { "1" } else { "0" }).unwrap_or("?");
    let ip = i.ipv4.first().cloned().unwrap_or_else(|| "-".to_string());
    let gw = i.gateway_v4.clone().unwrap_or_else(|| "-".to_string());
    let dns = if i.dns.is_empty() {
        "-".to_string()
    } else {
        i.dns.join(",")
    };

    format!(
        "state={}; carrier={}; ip={}; gw={}; dns={}",
        i.operstate, carrier, ip, gw, dns
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_prefers_ethernet_when_active() {
        let wifi = WifiState {
            ifaces: vec!["wlan0".to_string()],
            connected_ssid: Some("Home".to_string()),
            networks: vec![],
        };
        let ethernet = EthernetState {
            ifaces: vec![EthernetIface {
                name: "enp1s0".to_string(),
                operstate: "up".to_string(),
                carrier: Some(true),
                mac: None,
                speed_mbps: None,
                ipv4: vec!["192.168.1.2/24".to_string()],
                ipv6: vec![],
                gateway_v4: None,
                dns: vec![],
            }],
        };

        assert_eq!(
            determine_start_tab(StartupTabPolicy::PreferActive, &wifi, &ethernet),
            ActiveTab::Ethernet
        );
    }

    #[test]
    fn startup_falls_back_to_wifi_if_no_active_ethernet() {
        let wifi = WifiState {
            ifaces: vec!["wlan0".to_string()],
            connected_ssid: None,
            networks: vec![],
        };
        let ethernet = EthernetState { ifaces: vec![] };

        assert_eq!(
            determine_start_tab(StartupTabPolicy::PreferActive, &wifi, &ethernet),
            ActiveTab::Wifi
        );
    }
}
