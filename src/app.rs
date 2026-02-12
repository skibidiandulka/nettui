// Copyright (C) 2026 skibidiandulka
// Clean-room implementation inspired by Impala UX by pythops.

use crate::{
    backend::{iwd::IwdBackend, networkd::NetworkdBackend, traits::EthernetBackend},
    domain::{
        common::{ActiveTab, StartupTabPolicy, Toast, ToastKind, WifiFocus},
        ethernet::{EthernetIface, EthernetState},
        wifi::{WifiNetwork, WifiState},
    },
    keybinds::Keybinds,
};
use anyhow::Result;
use ratatui::widgets::TableState;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::task::JoinHandle;

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
    pub wifi_focus: WifiFocus,
    pub keybinds: Keybinds,

    pub wifi: WifiState,
    pub wifi_known_state: TableState,
    pub wifi_new_state: TableState,
    pub wifi_adapter_state: TableState,
    pub wifi_iface_details: Option<EthernetIface>,
    pub show_wifi_details: bool,
    pub show_unavailable_known_networks: bool,
    pub show_hidden_networks: bool,
    pub hidden_connect_prompt: bool,
    pub hidden_ssid_input: String,
    pub wifi_passphrase_prompt_ssid: Option<String>,
    pub wifi_passphrase_input: String,

    pub ethernet: EthernetState,
    pub ethernet_state: TableState,

    pub last_error: Option<String>,
    pub last_action: Option<String>,
    pub toast: Option<Toast>,
    pub wifi_scan_pending: bool,
    pub wifi_connect_pending: bool,
    pub spinner_frame: usize,

    wifi_backend: IwdBackend,
    eth_backend: NetworkdBackend,
    wifi_scan_task: Option<JoinHandle<Result<()>>>,
    wifi_connect_task: Option<JoinHandle<Result<()>>>,
    wifi_connect_context: Option<WifiConnectContext>,
}

#[derive(Debug, Clone)]
struct WifiConnectContext {
    ssid: String,
    disconnect: bool,
    used_passphrase: bool,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let wifi_backend = IwdBackend::new();
        let eth_backend = NetworkdBackend::new();

        let wifi = wifi_backend
            .query_state()
            .await
            .unwrap_or_else(|_| WifiState::empty());
        let ethernet = EthernetState {
            ifaces: eth_backend.list_ifaces().unwrap_or_default(),
        };
        let wifi_iface_details = wifi
            .ifaces
            .first()
            .and_then(|iface| eth_backend.iface_details(iface).ok());
        let active_tab = determine_start_tab(config.startup_policy, &wifi, &ethernet);

        let mut app = Self {
            running: true,
            config,
            active_tab,
            wifi_focus: WifiFocus::KnownNetworks,
            keybinds: Keybinds::load(),
            wifi,
            wifi_known_state: TableState::default(),
            wifi_new_state: TableState::default(),
            wifi_adapter_state: TableState::default(),
            wifi_iface_details,
            show_wifi_details: false,
            show_unavailable_known_networks: false,
            show_hidden_networks: false,
            hidden_connect_prompt: false,
            hidden_ssid_input: String::new(),
            wifi_passphrase_prompt_ssid: None,
            wifi_passphrase_input: String::new(),
            ethernet,
            ethernet_state: TableState::default(),
            last_error: None,
            last_action: None,
            toast: None,
            wifi_scan_pending: false,
            wifi_connect_pending: false,
            spinner_frame: 0,
            wifi_backend,
            eth_backend,
            wifi_scan_task: None,
            wifi_connect_task: None,
            wifi_connect_context: None,
        };

        app.init_wifi_states();
        app.init_ethernet_state();
        Ok(app)
    }

    pub async fn tick(&mut self) -> Result<()> {
        self.poll_background_tasks().await;
        self.spinner_frame = (self.spinner_frame + 1) % spinner_frames().len();

        if let Some(t) = &self.toast
            && Instant::now() >= t.until
        {
            self.toast = None;
        }
        self.refresh_all().await;
        Ok(())
    }

    async fn refresh_all(&mut self) {
        let known_ssid = self.selected_known_ssid();
        let new_ssid = self.selected_new_ssid();
        let selected_eth = self.selected_eth_iface().map(|i| i.name.clone());

        if let Ok(wifi) = self.wifi_backend.query_state().await {
            self.wifi = wifi;
            self.restore_wifi_selection(known_ssid, new_ssid);
            self.wifi_iface_details = self
                .wifi
                .ifaces
                .first()
                .and_then(|iface| self.eth_backend.iface_details(iface).ok());
        }

        if let Ok(ifaces) = self.eth_backend.list_ifaces() {
            self.ethernet = EthernetState { ifaces };
            self.restore_ethernet_selection(selected_eth);
        }

        self.ensure_valid_wifi_focus();
        self.last_error = None;
    }

    pub async fn refresh_current(&mut self) {
        self.refresh_all().await;
        self.set_toast(
            ToastKind::Info,
            format!(
                "Refreshed (Known: {}, New: {}, Ethernet: {})",
                self.known_total_len(),
                self.new_total_len(),
                self.ethernet.ifaces.len()
            ),
        );
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn switch_transport_next(&mut self) {
        self.active_tab = match self.active_tab {
            ActiveTab::Wifi => ActiveTab::Ethernet,
            ActiveTab::Ethernet => ActiveTab::Wifi,
        };
    }

    pub fn switch_transport_prev(&mut self) {
        self.switch_transport_next();
    }

    pub fn switch_focus_next(&mut self) {
        if self.active_tab != ActiveTab::Wifi {
            return;
        }

        for _ in 0..3 {
            self.wifi_focus = match self.wifi_focus {
                WifiFocus::KnownNetworks => WifiFocus::NewNetworks,
                WifiFocus::NewNetworks => WifiFocus::Adapter,
                WifiFocus::Adapter => WifiFocus::KnownNetworks,
            };
            if self.focus_has_items(self.wifi_focus) {
                return;
            }
        }
    }

    pub fn switch_focus_prev(&mut self) {
        if self.active_tab != ActiveTab::Wifi {
            return;
        }

        for _ in 0..3 {
            self.wifi_focus = match self.wifi_focus {
                WifiFocus::KnownNetworks => WifiFocus::Adapter,
                WifiFocus::NewNetworks => WifiFocus::KnownNetworks,
                WifiFocus::Adapter => WifiFocus::NewNetworks,
            };
            if self.focus_has_items(self.wifi_focus) {
                return;
            }
        }
    }

    pub fn select_next(&mut self) {
        match self.active_tab {
            ActiveTab::Wifi => match self.wifi_focus {
                WifiFocus::KnownNetworks => {
                    let len = self.known_total_len();
                    select_next_in_state(&mut self.wifi_known_state, len)
                }
                WifiFocus::NewNetworks => {
                    let len = self.new_total_len();
                    select_next_in_state(&mut self.wifi_new_state, len)
                }
                WifiFocus::Adapter => {
                    let len = self.device_total_len();
                    select_next_in_state(&mut self.wifi_adapter_state, len)
                }
            },
            ActiveTab::Ethernet => {
                select_next_in_state(&mut self.ethernet_state, self.ethernet.ifaces.len())
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.active_tab {
            ActiveTab::Wifi => match self.wifi_focus {
                WifiFocus::KnownNetworks => {
                    let len = self.known_total_len();
                    select_prev_in_state(&mut self.wifi_known_state, len)
                }
                WifiFocus::NewNetworks => {
                    let len = self.new_total_len();
                    select_prev_in_state(&mut self.wifi_new_state, len)
                }
                WifiFocus::Adapter => {
                    let len = self.device_total_len();
                    select_prev_in_state(&mut self.wifi_adapter_state, len)
                }
            },
            ActiveTab::Ethernet => {
                select_prev_in_state(&mut self.ethernet_state, self.ethernet.ifaces.len())
            }
        }
    }

    pub fn selected_wifi_network(&self) -> Option<&WifiNetwork> {
        match self.wifi_focus {
            WifiFocus::KnownNetworks => self.selected_known_network(),
            WifiFocus::NewNetworks => self.selected_new_network(),
            WifiFocus::Adapter => None,
        }
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

    pub fn toggle_wifi_details(&mut self) {
        if !self.wifi.has_adapter() {
            self.set_toast(ToastKind::Error, "No Wi-Fi adapter found");
            return;
        }
        self.show_wifi_details = !self.show_wifi_details;
    }

    pub fn toggle_known_show_all(&mut self) {
        self.show_unavailable_known_networks = !self.show_unavailable_known_networks;
        let len = self.known_total_len();
        clamp_selected(&mut self.wifi_known_state, len);
        let state = if self.show_unavailable_known_networks {
            "enabled"
        } else {
            "disabled"
        };
        self.set_toast(
            ToastKind::Info,
            format!(
                "Known: show all {state} ({} entries)",
                self.known_total_len()
            ),
        );
    }

    pub fn toggle_new_show_all(&mut self) {
        self.show_hidden_networks = !self.show_hidden_networks;
        let len = self.new_total_len();
        clamp_selected(&mut self.wifi_new_state, len);
        let state = if self.show_hidden_networks {
            "enabled"
        } else {
            "disabled"
        };
        self.set_toast(
            ToastKind::Info,
            format!("New: show all {state} ({} entries)", self.new_total_len()),
        );
    }

    pub fn open_hidden_connect_prompt(&mut self) {
        self.hidden_connect_prompt = true;
        self.hidden_ssid_input.clear();
    }

    pub fn close_hidden_connect_prompt(&mut self) {
        self.hidden_connect_prompt = false;
        self.hidden_ssid_input.clear();
    }

    pub fn hidden_input_push(&mut self, c: char) {
        self.hidden_ssid_input.push(c);
    }

    pub fn hidden_input_backspace(&mut self) {
        self.hidden_ssid_input.pop();
    }

    pub fn open_wifi_passphrase_prompt(&mut self, ssid: String) {
        self.wifi_passphrase_prompt_ssid = Some(ssid);
        self.wifi_passphrase_input.clear();
    }

    pub fn close_wifi_passphrase_prompt(&mut self) {
        self.wifi_passphrase_prompt_ssid = None;
        self.wifi_passphrase_input.clear();
    }

    pub fn passphrase_input_push(&mut self, c: char) {
        self.wifi_passphrase_input.push(c);
    }

    pub fn passphrase_input_backspace(&mut self) {
        self.wifi_passphrase_input.pop();
    }

    pub async fn submit_hidden_connect(&mut self) {
        let ssid = self.hidden_ssid_input.trim().to_string();
        if ssid.is_empty() {
            self.set_toast(ToastKind::Error, "SSID cannot be empty");
            return;
        }

        match self.wifi_backend.connect_hidden(&ssid).await {
            Ok(()) => {
                self.last_action = Some(format!("Connect hidden requested: {ssid}"));
                self.set_toast(ToastKind::Info, format!("Connect hidden requested: {ssid}"));
                self.notify("Wi-Fi", &format!("Connect hidden: {ssid}"))
                    .await;
                self.close_hidden_connect_prompt();
            }
            Err(e) => {
                let msg = friendly_wifi_error("connect hidden network", &e);
                self.set_toast(ToastKind::Error, msg);
            }
        }
    }

    pub async fn submit_wifi_passphrase_connect(&mut self) {
        let Some(ssid) = self.wifi_passphrase_prompt_ssid.clone() else {
            return;
        };
        let passphrase = self.wifi_passphrase_input.clone();
        if passphrase.is_empty() {
            self.set_toast(ToastKind::Error, "Passphrase cannot be empty");
            return;
        }

        if self.wifi_connect_pending {
            self.set_toast(ToastKind::Info, "Wi-Fi connect already in progress");
            return;
        }

        self.last_action = Some(format!("Connecting to {ssid}..."));
        self.set_toast(ToastKind::Info, format!("Connecting to {ssid}..."));
        self.close_wifi_passphrase_prompt();

        self.wifi_connect_pending = true;
        self.wifi_connect_context = Some(WifiConnectContext {
            ssid: ssid.clone(),
            disconnect: false,
            used_passphrase: true,
        });
        self.wifi_connect_task = Some(tokio::spawn(async move {
            IwdBackend::new()
                .connect_with_passphrase(&ssid, &passphrase)
                .await
        }));
    }

    pub async fn notify(&self, title: &str, body: &str) {
        let _ = Command::new("notify-send")
            .arg(title)
            .arg(body)
            .arg("-t")
            .arg("2200")
            .output()
            .await;
    }

    pub async fn wifi_scan(&mut self) -> Result<()> {
        if self.wifi_scan_pending {
            self.set_toast(ToastKind::Info, "Wi-Fi scan already running");
            return Ok(());
        }

        self.wifi_scan_pending = true;
        self.last_action = Some("Wi-Fi scan requested".to_string());
        self.set_toast(ToastKind::Info, "Wi-Fi scan requested...");
        self.wifi_scan_task = Some(tokio::spawn(async { IwdBackend::new().scan().await }));
        Ok(())
    }

    pub async fn wifi_connect_or_disconnect(&mut self) -> Result<()> {
        if self.wifi_connect_pending {
            self.set_toast(
                ToastKind::Info,
                "Wi-Fi connect/disconnect already in progress",
            );
            return Ok(());
        }

        let Some(net) = self.selected_wifi_network().cloned() else {
            self.set_toast(ToastKind::Error, "No network selected");
            return Ok(());
        };

        if self.wifi_focus == WifiFocus::KnownNetworks && !net.available {
            self.set_toast(
                ToastKind::Info,
                "Unavailable known network cannot be connected directly",
            );
            return Ok(());
        }

        let ssid = net.ssid.clone();
        let disconnect = net.connected;

        self.wifi_connect_pending = true;
        self.wifi_connect_context = Some(WifiConnectContext {
            ssid: ssid.clone(),
            disconnect,
            used_passphrase: false,
        });
        self.last_action = Some(if disconnect {
            "Disconnecting Wi-Fi...".to_string()
        } else {
            format!("Connecting to {ssid}...")
        });
        self.set_toast(
            ToastKind::Info,
            if disconnect {
                "Disconnecting Wi-Fi...".to_string()
            } else {
                format!("Connecting to {ssid}...")
            },
        );

        self.wifi_connect_task = Some(tokio::spawn(async move {
            if disconnect {
                IwdBackend::new().disconnect().await
            } else {
                IwdBackend::new().connect(&ssid).await
            }
        }));

        Ok(())
    }

    pub async fn wifi_forget_selected(&mut self) -> Result<()> {
        if self.wifi_focus != WifiFocus::KnownNetworks {
            self.set_toast(ToastKind::Info, "Forget is available in Known Networks");
            return Ok(());
        }

        let Some(net) = self.selected_known_network().cloned() else {
            self.set_toast(ToastKind::Error, "No known network selected");
            return Ok(());
        };

        match self.wifi_backend.forget_known(&net.ssid).await {
            Ok(()) => {
                self.last_action = Some(format!("Forgot network {}", net.ssid));
                self.set_toast(ToastKind::Success, format!("Forgot network {}", net.ssid));
                self.notify("Wi-Fi", &format!("Forgot network {}", net.ssid))
                    .await;
            }
            Err(e) => {
                let msg = friendly_wifi_error("forget known network", &e);
                self.set_toast(ToastKind::Error, msg);
            }
        }
        Ok(())
    }

    pub async fn wifi_toggle_autoconnect_selected(&mut self) -> Result<()> {
        if self.wifi_focus != WifiFocus::KnownNetworks {
            self.set_toast(
                ToastKind::Info,
                "Autoconnect toggle is available in Known Networks",
            );
            return Ok(());
        }

        let Some(net) = self.selected_known_network().cloned() else {
            self.set_toast(ToastKind::Error, "No known network selected");
            return Ok(());
        };

        match self.wifi_backend.toggle_autoconnect(&net.ssid).await {
            Ok(enabled) => {
                let state = if enabled { "enabled" } else { "disabled" };
                self.last_action = Some(format!("Autoconnect {} for {}", state, net.ssid));
                self.set_toast(
                    ToastKind::Success,
                    format!("Autoconnect {} for {}", state, net.ssid),
                );
                self.notify("Wi-Fi", &format!("Autoconnect {}: {}", state, net.ssid))
                    .await;
            }
            Err(e) => {
                let msg = friendly_wifi_error("toggle autoconnect", &e);
                self.set_toast(ToastKind::Error, msg);
            }
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
            msg.push_str(" (elevated)");
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

    pub async fn ethernet_toggle_link(&mut self) -> Result<()> {
        let iface = self
            .selected_eth_iface()
            .cloned()
            .ok_or_else(|| std::io::Error::other("no ethernet interface selected"))?;
        let target_up = !(iface.operstate == "up" || iface.carrier == Some(true));
        let state_word = if target_up { "up" } else { "down" };

        let out = self
            .eth_backend
            .set_link_admin_state(&iface.name, target_up)
            .await?;
        self.refresh_all().await;

        let mut msg = format!("{}: link set {}", iface.name, state_word);
        if out.used_sudo {
            msg.push_str(" (elevated)");
        }
        if !out.stderr.is_empty() {
            msg.push_str(&format!("\nstderr: {}", out.stderr));
        }
        self.last_action = Some(format!("{} link {}", iface.name, state_word));
        self.set_toast(ToastKind::Success, msg);
        self.notify("Ethernet", &format!("{} link {}", iface.name, state_word))
            .await;
        Ok(())
    }

    pub fn wifi_scanning_active(&self) -> bool {
        self.wifi_scan_pending
    }

    pub fn wifi_connect_active(&self) -> bool {
        self.wifi_connect_pending
    }

    pub fn spinner_glyph(&self) -> &'static str {
        spinner_frames()[self.spinner_frame % spinner_frames().len()]
    }

    async fn poll_background_tasks(&mut self) {
        if let Some(handle) = self.wifi_scan_task.take() {
            if handle.is_finished() {
                self.wifi_scan_pending = false;
                match handle.await {
                    Ok(Ok(())) => {
                        self.last_action = Some("Wi-Fi scan completed".to_string());
                        self.set_toast(ToastKind::Success, "Wi-Fi scan completed");
                        self.notify("Wi-Fi", "Scan completed").await;
                    }
                    Ok(Err(e)) => {
                        let msg = friendly_wifi_error("scan", &e);
                        self.set_toast(ToastKind::Error, msg);
                    }
                    Err(e) => {
                        self.set_toast(ToastKind::Error, format!("scan task failed: {e}"));
                    }
                }
            } else {
                self.wifi_scan_task = Some(handle);
            }
        }

        if let Some(handle) = self.wifi_connect_task.take() {
            if handle.is_finished() {
                self.wifi_connect_pending = false;
                let ctx = self.wifi_connect_context.take();
                match handle.await {
                    Ok(Ok(())) => {
                        if let Some(ctx) = ctx {
                            if ctx.disconnect {
                                self.last_action = Some("Disconnected Wi-Fi".to_string());
                                self.set_toast(
                                    ToastKind::Success,
                                    format!("Disconnected from {}", ctx.ssid),
                                );
                                self.notify("Wi-Fi", &format!("Disconnected from {}", ctx.ssid))
                                    .await;
                            } else {
                                self.last_action = Some(format!("Connected to {}", ctx.ssid));
                                self.set_toast(
                                    ToastKind::Success,
                                    format!("Connected to {}", ctx.ssid),
                                );
                                self.notify("Wi-Fi", &format!("Connected to {}", ctx.ssid))
                                    .await;
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        let no_agent = is_no_agent_error(&e);
                        if let Some(ctx) = ctx
                            && no_agent
                            && !ctx.disconnect
                            && !ctx.used_passphrase
                        {
                            self.open_wifi_passphrase_prompt(ctx.ssid.clone());
                            self.set_toast(
                                ToastKind::Info,
                                format!("Passphrase required for {}", ctx.ssid),
                            );
                        } else {
                            let msg = friendly_wifi_error("connect/disconnect", &e);
                            self.set_toast(ToastKind::Error, msg);
                        }
                    }
                    Err(e) => {
                        self.set_toast(ToastKind::Error, format!("connect task failed: {e}"));
                    }
                }
            } else {
                self.wifi_connect_task = Some(handle);
            }
        }
    }

    fn known_total_len(&self) -> usize {
        self.wifi.known_networks.len()
            + if self.show_unavailable_known_networks {
                self.wifi.unavailable_known_networks.len()
            } else {
                0
            }
    }

    fn new_total_len(&self) -> usize {
        self.wifi.new_networks.len()
            + if self.show_hidden_networks {
                self.wifi.hidden_networks.len()
            } else {
                0
            }
    }

    fn device_total_len(&self) -> usize {
        usize::from(self.wifi.device.is_some())
    }

    fn selected_known_network(&self) -> Option<&WifiNetwork> {
        let idx = self.wifi_known_state.selected()?;
        if idx < self.wifi.known_networks.len() {
            return self.wifi.known_networks.get(idx);
        }
        if !self.show_unavailable_known_networks {
            return None;
        }
        let hidden_idx = idx.saturating_sub(self.wifi.known_networks.len());
        self.wifi.unavailable_known_networks.get(hidden_idx)
    }

    fn selected_new_network(&self) -> Option<&WifiNetwork> {
        let idx = self.wifi_new_state.selected()?;
        if idx < self.wifi.new_networks.len() {
            return self.wifi.new_networks.get(idx);
        }
        if !self.show_hidden_networks {
            return None;
        }
        let hidden_idx = idx.saturating_sub(self.wifi.new_networks.len());
        self.wifi.hidden_networks.get(hidden_idx)
    }

    fn init_wifi_states(&mut self) {
        let known_len = self.known_total_len();
        let new_len = self.new_total_len();
        let device_len = self.device_total_len();
        select_first_if_any(&mut self.wifi_known_state, known_len);
        select_first_if_any(&mut self.wifi_new_state, new_len);
        select_first_if_any(&mut self.wifi_adapter_state, device_len);
        self.ensure_valid_wifi_focus();
    }

    fn init_ethernet_state(&mut self) {
        select_first_if_any(&mut self.ethernet_state, self.ethernet.ifaces.len());
    }

    fn selected_known_ssid(&self) -> Option<String> {
        self.selected_known_network().map(|n| n.ssid.clone())
    }

    fn selected_new_ssid(&self) -> Option<String> {
        self.selected_new_network().map(|n| n.ssid.clone())
    }

    fn restore_wifi_selection(&mut self, known_ssid: Option<String>, new_ssid: Option<String>) {
        if let Some(ssid) = known_ssid {
            if let Some(idx) = self.wifi.known_networks.iter().position(|n| n.ssid == ssid) {
                self.wifi_known_state.select(Some(idx));
            } else if self.show_unavailable_known_networks {
                if let Some(idx) = self
                    .wifi
                    .unavailable_known_networks
                    .iter()
                    .position(|n| n.ssid == ssid)
                {
                    self.wifi_known_state
                        .select(Some(self.wifi.known_networks.len() + idx));
                } else {
                    let len = self.known_total_len();
                    select_first_if_any(&mut self.wifi_known_state, len);
                }
            } else {
                let len = self.known_total_len();
                select_first_if_any(&mut self.wifi_known_state, len);
            }
        } else {
            let len = self.known_total_len();
            select_first_if_any(&mut self.wifi_known_state, len);
        }

        if let Some(ssid) = new_ssid {
            if let Some(idx) = self.wifi.new_networks.iter().position(|n| n.ssid == ssid) {
                self.wifi_new_state.select(Some(idx));
            } else if self.show_hidden_networks {
                if let Some(idx) = self
                    .wifi
                    .hidden_networks
                    .iter()
                    .position(|n| n.ssid == ssid)
                {
                    self.wifi_new_state
                        .select(Some(self.wifi.new_networks.len() + idx));
                } else {
                    let len = self.new_total_len();
                    select_first_if_any(&mut self.wifi_new_state, len);
                }
            } else {
                let len = self.new_total_len();
                select_first_if_any(&mut self.wifi_new_state, len);
            }
        } else {
            let len = self.new_total_len();
            select_first_if_any(&mut self.wifi_new_state, len);
        }

        let len = self.device_total_len();
        select_first_if_any(&mut self.wifi_adapter_state, len);
    }

    fn restore_ethernet_selection(&mut self, selected_iface: Option<String>) {
        if let Some(name) = selected_iface
            && let Some(idx) = self.ethernet.ifaces.iter().position(|i| i.name == name)
        {
            self.ethernet_state.select(Some(idx));
            return;
        }
        select_first_if_any(&mut self.ethernet_state, self.ethernet.ifaces.len());
    }

    fn ensure_valid_wifi_focus(&mut self) {
        if self.focus_has_items(self.wifi_focus) {
            return;
        }
        for candidate in [
            WifiFocus::KnownNetworks,
            WifiFocus::NewNetworks,
            WifiFocus::Adapter,
        ] {
            if self.focus_has_items(candidate) {
                self.wifi_focus = candidate;
                return;
            }
        }
    }

    fn focus_has_items(&self, focus: WifiFocus) -> bool {
        match focus {
            WifiFocus::KnownNetworks => self.known_total_len() > 0,
            WifiFocus::NewNetworks => self.new_total_len() > 0,
            WifiFocus::Adapter => self.device_total_len() > 0,
        }
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
            } else if wifi.is_active() || wifi.has_adapter() {
                ActiveTab::Wifi
            } else {
                ActiveTab::Ethernet
            }
        }
    }
}

fn friendly_wifi_error(action: &str, err: &anyhow::Error) -> String {
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    if lower.contains("no agent registered") {
        return format!(
            "{} needs Wi-Fi credentials. Use connect again and enter passphrase.",
            action
        );
    }
    if lower.contains("accessdenied")
        || lower.contains("permission denied")
        || lower.contains("not authorized")
        || lower.contains("operation not permitted")
    {
        return format!(
            "{} requires elevated permissions. Run with proper privileges and retry.",
            action
        );
    }
    msg
}

fn is_no_agent_error(err: &anyhow::Error) -> bool {
    err.to_string()
        .to_lowercase()
        .contains("no agent registered")
}

fn spinner_frames() -> &'static [&'static str] {
    &["-", "\\", "|", "/"]
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

fn select_first_if_any(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
    } else if state.selected().is_none() {
        state.select(Some(0));
    }
}

fn select_next_in_state(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let i = match state.selected() {
        Some(i) => (i + 1).min(len - 1),
        None => 0,
    };
    state.select(Some(i));
}

fn select_prev_in_state(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let i = match state.selected() {
        Some(i) => i.saturating_sub(1),
        None => 0,
    };
    state.select(Some(i));
}

fn clamp_selected(state: &mut TableState, len: usize) {
    if len == 0 {
        state.select(None);
        return;
    }
    let idx = state.selected().unwrap_or(0).min(len - 1);
    state.select(Some(idx));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_prefers_ethernet_when_active() {
        let wifi = WifiState {
            ifaces: vec!["wlan0".to_string()],
            connected_ssid: Some("Home".to_string()),
            known_networks: vec![],
            unavailable_known_networks: vec![],
            new_networks: vec![],
            hidden_networks: vec![],
            device: None,
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
            known_networks: vec![],
            unavailable_known_networks: vec![],
            new_networks: vec![],
            hidden_networks: vec![],
            device: None,
        };
        let ethernet = EthernetState { ifaces: vec![] };

        assert_eq!(
            determine_start_tab(StartupTabPolicy::PreferActive, &wifi, &ethernet),
            ActiveTab::Wifi
        );
    }
}
