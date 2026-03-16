// Copyright (C) 2026 skibidiandulka
// Clean-room implementation inspired by Impala UX by pythops.

mod actions;
mod background;
mod state;

use crate::{
    backend::{
        iwd::{
            AuthAgentEvent, AuthPromptRequest, AuthPromptRequestKind, AuthPromptResponse,
            IwdBackend, WifiShareCredentials,
        },
        networkd::NetworkdBackend,
        traits::EthernetBackend,
    },
    domain::{
        common::{ActiveTab, StartupTabPolicy, Toast, ToastKind, WifiFocus},
        ethernet::{EthernetIface, EthernetState},
        wifi::{WifiNetwork, WifiState},
    },
    keybinds::Keybinds,
};
use anyhow::Result;
use iwdrs::agent::AgentManager;
use ratatui::widgets::TableState;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use tokio::process::Command;
use tokio::sync::{mpsc::UnboundedReceiver, oneshot};
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy)]
pub struct AppConfig {
    pub startup_policy: StartupTabPolicy,
    pub tick_ms: u64,
    pub data_refresh_ms: u64,
    pub job_timeout_scan_ms: u64,
    pub job_timeout_connect_ms: u64,
    pub scan_debounce_ms: u64,
    pub esc_quit: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            startup_policy: StartupTabPolicy::PreferActive,
            tick_ms: 250,
            data_refresh_ms: 900,
            job_timeout_scan_ms: 12_000,
            job_timeout_connect_ms: 20_000,
            scan_debounce_ms: 700,
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
    pub wifi_auth_prompt: Option<WifiAuthPrompt>,
    pub wifi_ap_prompt_open: bool,
    pub wifi_ap_ssid_input: String,
    pub wifi_ap_passphrase_input: String,
    pub wifi_ap_passphrase_visible: bool,
    pub wifi_ap_prompt_field: WifiApPromptField,
    pub wifi_share_popup: Option<WifiSharePopup>,

    pub ethernet: EthernetState,
    pub ethernet_state: TableState,

    pub last_error: Option<String>,
    pub last_action: Option<String>,
    pub toasts: VecDeque<Toast>,
    pub wifi_scan_pending: bool,
    pub wifi_connect_pending: bool,
    pub wifi_ap_pending: bool,
    last_data_refresh_at: Instant,
    refresh_requested: bool,
    wifi_scan_started_at: Option<Instant>,
    wifi_connect_started_at: Option<Instant>,
    wifi_ap_started_at: Option<Instant>,
    last_scan_request_at: Option<Instant>,

    wifi_backend: IwdBackend,
    eth_backend: NetworkdBackend,
    wifi_auth_agent: Option<AgentManager>,
    wifi_auth_events: Option<UnboundedReceiver<AuthAgentEvent>>,
    wifi_scan_task: Option<JoinHandle<Result<()>>>,
    wifi_connect_task: Option<JoinHandle<Result<()>>>,
    wifi_ap_task: Option<JoinHandle<Result<()>>>,
    wifi_connect_context: Option<WifiConnectContext>,
    wifi_ap_context: Option<WifiApContext>,
}

#[derive(Debug, Clone)]
struct WifiConnectContext {
    ssid: String,
    disconnect: bool,
    used_passphrase: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiAuthPromptField {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub enum WifiAuthPromptKind {
    ManualPassphrase,
    AgentPassphrase,
    AgentPrivateKeyPassphrase,
    AgentUserPassword { user_name: Option<String> },
    AgentUserNameAndPassphrase,
}

pub struct WifiAuthPrompt {
    pub ssid: String,
    pub kind: WifiAuthPromptKind,
    pub primary_input: String,
    pub secondary_input: String,
    pub focus: WifiAuthPromptField,
    pub primary_visible: bool,
    pub secondary_visible: bool,
    responder: Option<oneshot::Sender<AuthPromptResponse>>,
}

impl WifiAuthPrompt {
    fn manual_passphrase(ssid: String) -> Self {
        Self {
            ssid,
            kind: WifiAuthPromptKind::ManualPassphrase,
            primary_input: String::new(),
            secondary_input: String::new(),
            focus: WifiAuthPromptField::Primary,
            primary_visible: false,
            secondary_visible: false,
            responder: None,
        }
    }

    fn from_agent_request(request: AuthPromptRequest) -> Self {
        let (ssid, kind, responder) = request.into_parts();
        let kind = match kind {
            AuthPromptRequestKind::Passphrase => WifiAuthPromptKind::AgentPassphrase,
            AuthPromptRequestKind::PrivateKeyPassphrase => {
                WifiAuthPromptKind::AgentPrivateKeyPassphrase
            }
            AuthPromptRequestKind::UserPassword { user_name } => {
                WifiAuthPromptKind::AgentUserPassword { user_name }
            }
            AuthPromptRequestKind::UserNameAndPassphrase => {
                WifiAuthPromptKind::AgentUserNameAndPassphrase
            }
        };

        Self {
            ssid,
            kind,
            primary_input: String::new(),
            secondary_input: String::new(),
            focus: WifiAuthPromptField::Primary,
            primary_visible: false,
            secondary_visible: false,
            responder: Some(responder),
        }
    }

    pub fn title(&self) -> &'static str {
        match self.kind {
            WifiAuthPromptKind::ManualPassphrase | WifiAuthPromptKind::AgentPassphrase => {
                " Wi-Fi Passphrase "
            }
            WifiAuthPromptKind::AgentPrivateKeyPassphrase => " Private Key Passphrase ",
            WifiAuthPromptKind::AgentUserPassword { .. } => " Wi-Fi Password ",
            WifiAuthPromptKind::AgentUserNameAndPassphrase => " Wi-Fi Credentials ",
        }
    }

    pub fn primary_label(&self) -> &'static str {
        match self.kind {
            WifiAuthPromptKind::ManualPassphrase | WifiAuthPromptKind::AgentPassphrase => {
                "Passphrase"
            }
            WifiAuthPromptKind::AgentPrivateKeyPassphrase => "Key passphrase",
            WifiAuthPromptKind::AgentUserPassword { .. } => "Password",
            WifiAuthPromptKind::AgentUserNameAndPassphrase => "Username",
        }
    }

    pub fn secondary_label(&self) -> Option<&'static str> {
        match self.kind {
            WifiAuthPromptKind::AgentUserNameAndPassphrase => Some("Password"),
            _ => None,
        }
    }

    pub fn primary_is_secret(&self) -> bool {
        !matches!(self.kind, WifiAuthPromptKind::AgentUserNameAndPassphrase)
    }

    pub fn secondary_is_secret(&self) -> bool {
        self.secondary_label().is_some()
    }

    pub fn fixed_user_name(&self) -> Option<&str> {
        match &self.kind {
            WifiAuthPromptKind::AgentUserPassword {
                user_name: Some(user_name),
            } => Some(user_name.as_str()),
            _ => None,
        }
    }

    pub fn has_secondary_field(&self) -> bool {
        self.secondary_label().is_some()
    }

    pub fn status_message(&self) -> String {
        match self.kind {
            WifiAuthPromptKind::ManualPassphrase
            | WifiAuthPromptKind::AgentPassphrase
            | WifiAuthPromptKind::AgentPrivateKeyPassphrase => {
                format!("Credentials required for {}", self.ssid)
            }
            WifiAuthPromptKind::AgentUserPassword { .. }
            | WifiAuthPromptKind::AgentUserNameAndPassphrase => {
                format!("Enterprise authentication required for {}", self.ssid)
            }
        }
    }

    fn cancel(mut self) {
        if let Some(responder) = self.responder.take() {
            let _ = responder.send(AuthPromptResponse::Cancel);
        }
    }

    fn take_responder(&mut self) -> Option<oneshot::Sender<AuthPromptResponse>> {
        self.responder.take()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiApPromptField {
    Ssid,
    Passphrase,
}

#[derive(Debug, Clone)]
struct WifiApContext {
    ssid: Option<String>,
    stopping: bool,
}

#[derive(Debug, Clone)]
pub struct WifiSharePopup {
    pub ssid: String,
    pub passphrase: String,
    pub qr_text: String,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let now = Instant::now();
        let wifi_backend = IwdBackend::new();
        let eth_backend = NetworkdBackend::new();

        let (wifi, wifi_start_error) = match wifi_backend.query_state().await {
            Ok(wifi) => (wifi, None),
            Err(err) => (WifiState::empty(), Some(err)),
        };
        let ethernet = EthernetState {
            ifaces: eth_backend.list_ifaces().unwrap_or_default(),
        };
        let wifi_iface_details = wifi
            .device
            .as_ref()
            .map(|device| device.iface.clone())
            .and_then(|iface| eth_backend.iface_details(&iface).ok());
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
            wifi_auth_prompt: None,
            wifi_ap_prompt_open: false,
            wifi_ap_ssid_input: String::new(),
            wifi_ap_passphrase_input: String::new(),
            wifi_ap_passphrase_visible: false,
            wifi_ap_prompt_field: WifiApPromptField::Ssid,
            wifi_share_popup: None,
            ethernet,
            ethernet_state: TableState::default(),
            last_error: None,
            last_action: None,
            toasts: VecDeque::new(),
            wifi_scan_pending: false,
            wifi_connect_pending: false,
            wifi_ap_pending: false,
            last_data_refresh_at: now,
            refresh_requested: false,
            wifi_scan_started_at: None,
            wifi_connect_started_at: None,
            wifi_ap_started_at: None,
            last_scan_request_at: None,
            wifi_backend,
            eth_backend,
            wifi_auth_agent: None,
            wifi_auth_events: None,
            wifi_scan_task: None,
            wifi_connect_task: None,
            wifi_ap_task: None,
            wifi_connect_context: None,
            wifi_ap_context: None,
        };

        app.init_wifi_states();
        app.init_ethernet_state();
        if let Some(err) = wifi_start_error {
            app.set_toast(
                ToastKind::Error,
                friendly_wifi_error("load Wi-Fi state", &err),
            );
        }
        if let Ok((agent_manager, auth_events)) = app.wifi_backend.register_auth_agent().await {
            app.wifi_auth_agent = Some(agent_manager);
            app.wifi_auth_events = Some(auth_events);
        }
        if let Some(msg) = detect_conflicting_wifi_services().await {
            app.last_action = Some(msg.clone());
            app.set_toast(ToastKind::Info, msg);
        }
        Ok(app)
    }

    pub async fn tick(&mut self) -> Result<()> {
        self.poll_auth_agent_events();
        self.poll_background_tasks().await;
        let now = Instant::now();

        while self
            .toasts
            .front()
            .is_some_and(|toast| toast.until.is_some_and(|until| now >= until))
        {
            self.toasts.pop_front();
        }
        if self.refresh_requested
            || refresh_due(self.last_data_refresh_at, self.config.data_refresh_ms, now)
        {
            self.refresh_all().await;
            self.refresh_requested = false;
            self.last_data_refresh_at = now;
        }
        Ok(())
    }

    async fn refresh_all(&mut self) {
        let known_ssid = self.selected_known_ssid();
        let new_ssid = self.selected_new_ssid();
        let selected_eth = self.selected_eth_iface().map(|i| i.name.clone());

        match self.wifi_backend.query_state().await {
            Ok(wifi) => {
                self.wifi = wifi;
                self.restore_wifi_selection(known_ssid, new_ssid);
                self.wifi_iface_details = self
                    .wifi
                    .device
                    .as_ref()
                    .map(|device| device.iface.clone())
                    .and_then(|iface| self.eth_backend.iface_details(&iface).ok());
            }
            Err(err) => {
                self.push_toast(
                    ToastKind::Error,
                    friendly_wifi_error("refresh Wi-Fi state", &err),
                );
            }
        }

        match self.eth_backend.list_ifaces() {
            Ok(ifaces) => {
                self.ethernet = EthernetState { ifaces };
                self.restore_ethernet_selection(selected_eth);
            }
            Err(err) => {
                self.push_toast(ToastKind::Error, shorten_banner_message(&err.to_string()));
            }
        }

        self.ensure_valid_wifi_focus();
        self.last_error = None;
    }

    pub async fn refresh_current(&mut self) {
        self.refresh_all().await;
        self.last_data_refresh_at = Instant::now();
        self.refresh_requested = false;
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

    fn request_refresh(&mut self) {
        self.refresh_requested = true;
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
        self.push_toast(kind, msg);
    }

    pub fn visible_toasts(&self) -> Vec<Toast> {
        let mut visible = self
            .toasts
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect::<Vec<_>>();

        if let Some(status) = self.wifi_status_toast()
            && !visible.iter().any(|toast| toast.msg == status.msg)
        {
            visible.insert(0, status);
            visible.truncate(3);
        }

        visible
    }

    fn wifi_status_toast(&self) -> Option<Toast> {
        if self.wifi_scan_pending {
            return Some(Toast {
                kind: ToastKind::Info,
                msg: "Scanning for Wi-Fi networks".to_string(),
                until: None,
            });
        }

        if self.wifi_ap_prompt_open {
            return Some(Toast {
                kind: ToastKind::Info,
                msg: "Set up Wi-Fi access point".to_string(),
                until: None,
            });
        }

        if let Some(prompt) = &self.wifi_auth_prompt {
            return Some(Toast {
                kind: ToastKind::Info,
                msg: prompt.status_message(),
                until: None,
            });
        }

        if self.wifi_connect_pending {
            let msg = self
                .wifi_connect_status_label()
                .unwrap_or_else(|| "Connecting to Wi-Fi".to_string());
            return Some(Toast {
                kind: ToastKind::Info,
                msg,
                until: None,
            });
        }

        if self.wifi_ap_pending {
            let msg = self
                .wifi_ap_status_label()
                .unwrap_or_else(|| "Changing access point mode".to_string());
            return Some(Toast {
                kind: ToastKind::Info,
                msg,
                until: None,
            });
        }

        None
    }

    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    pub fn block_if_busy(&mut self, attempted_action: &str) -> bool {
        let Some(active) = self.active_operation_label() else {
            return false;
        };
        self.set_toast(
            ToastKind::Info,
            format!("{active} is still running. Wait before {attempted_action}."),
        );
        true
    }

    fn active_operation_label(&self) -> Option<String> {
        if self.wifi_connect_pending {
            return Some(
                self.wifi_connect_status_label()
                    .unwrap_or_else(|| "Wi-Fi connection change".to_string()),
            );
        }

        if self.wifi_ap_pending {
            return Some(
                self.wifi_ap_status_label()
                    .unwrap_or_else(|| "Access point change".to_string()),
            );
        }

        if self.wifi_scan_pending {
            return Some("Wi-Fi scan".to_string());
        }

        None
    }

    fn wifi_connect_status_label(&self) -> Option<String> {
        self.wifi_connect_context.as_ref().map(|ctx| {
            if ctx.disconnect {
                format!("Disconnecting from {}", ctx.ssid)
            } else {
                format!("Connecting to {}", ctx.ssid)
            }
        })
    }

    fn wifi_ap_status_label(&self) -> Option<String> {
        self.wifi_ap_context.as_ref().map(|ctx| {
            if ctx.stopping {
                "Stopping access point".to_string()
            } else {
                format!(
                    "Starting access point {}",
                    ctx.ssid.clone().unwrap_or_else(|| "hotspot".to_string())
                )
            }
        })
    }

    fn push_toast(&mut self, kind: ToastKind, msg: impl Into<String>) {
        let msg = msg.into();
        if self.toasts.iter().any(|toast| {
            toast.msg == msg && std::mem::discriminant(&toast.kind) == std::mem::discriminant(&kind)
        }) {
            return;
        }
        self.toasts.push_back(Toast {
            kind,
            msg,
            until: Some(Instant::now() + Duration::from_millis(3000)),
        });
        while self.toasts.len() > 12 {
            self.toasts.pop_front();
        }
    }

    fn has_recent_toast_message(&self, needle: &str) -> bool {
        self.toasts
            .iter()
            .rev()
            .take(3)
            .any(|toast| toast.msg.contains(needle))
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

async fn detect_conflicting_wifi_services() -> Option<String> {
    let mut active = Vec::new();
    if is_service_active("NetworkManager.service").await {
        active.push("NetworkManager");
    }
    if is_service_active("wpa_supplicant.service").await {
        active.push("wpa_supplicant");
    }
    if active.is_empty() {
        None
    } else {
        Some(format!(
            "Detected active Wi-Fi manager(s): {}. iwd backend works best without overlapping Wi-Fi managers.",
            active.join(", ")
        ))
    }
}

async fn is_service_active(unit: &str) -> bool {
    match Command::new("systemctl")
        .arg("is-active")
        .arg("--quiet")
        .arg(unit)
        .status()
        .await
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

fn refresh_due(last_refresh: Instant, interval_ms: u64, now: Instant) -> bool {
    now.duration_since(last_refresh) >= Duration::from_millis(interval_ms)
}

fn scan_debounce_remaining(
    last_request: Option<Instant>,
    debounce_ms: u64,
    now: Instant,
) -> Option<Duration> {
    if debounce_ms == 0 {
        return None;
    }
    let last = last_request?;
    let elapsed = now.duration_since(last);
    let debounce = Duration::from_millis(debounce_ms);
    if elapsed >= debounce {
        None
    } else {
        Some(debounce - elapsed)
    }
}

fn friendly_wifi_error(action: &str, err: &anyhow::Error) -> String {
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    if lower.contains("argument format is invalid")
        || lower.contains("invalid passphrase")
        || lower.contains("invalid psk")
        || lower.contains("pre-shared key")
        || lower.contains("4-way handshake")
        || lower.contains("authentication failed")
        || lower.contains("bad passphrase")
        || lower.contains("wrong password")
    {
        return "Couldn't connect. Check the password and try again.".to_string();
    }
    if lower.contains("network not found") {
        return "Couldn't find that Wi-Fi network.".to_string();
    }
    if lower.contains("not supported") || lower.contains("operation not available") {
        return "This Wi-Fi adapter does not support that operation.".to_string();
    }
    if lower.contains("already exists") {
        return "That access point is already active.".to_string();
    }
    if lower.contains("no wifi station found") || lower.contains("no wifi adapter found") {
        return "No Wi-Fi adapter found.".to_string();
    }
    if lower.contains("no access point interface found") {
        return "This Wi-Fi adapter cannot switch to access point mode.".to_string();
    }
    if lower.contains("org.freedesktop.dbus.error.noreply")
        || lower.contains("remote peer disconnected")
        || lower.contains("connection reset by peer")
    {
        return "Couldn't talk to iwd. Restart nettui and try again.".to_string();
    }
    if lower.contains("cannot access iwd service") {
        return "Couldn't talk to iwd. Make sure the service is running.".to_string();
    }
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
    shorten_banner_message(&msg)
}

fn is_no_agent_error(err: &anyhow::Error) -> bool {
    err.to_string()
        .to_lowercase()
        .contains("no agent registered")
}

fn is_agent_canceled_error(err: &anyhow::Error) -> bool {
    let lower = err.to_string().to_lowercase();
    lower.contains("user canceled")
        || lower.contains("user-canceled")
        || lower.contains("operation canceled")
        || lower.contains("cancelled")
        || lower == "canceled"
}

fn shorten_banner_message(msg: &str) -> String {
    let single_line = msg
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let compact = single_line.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut shortened = compact;
    if shortened.len() > 120 {
        shortened.truncate(117);
        shortened.push_str("...");
    }
    shortened
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
            station_iface: Some("wlan0".to_string()),
            access_point_iface: None,
            connected_ssid: Some("Home".to_string()),
            access_point_ssid: None,
            access_point_clients: vec![],
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
            station_iface: Some("wlan0".to_string()),
            access_point_iface: None,
            connected_ssid: None,
            access_point_ssid: None,
            access_point_clients: vec![],
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

    #[test]
    fn refresh_due_respects_interval() {
        let base = Instant::now();
        assert!(!refresh_due(base, 900, base + Duration::from_millis(300)));
        assert!(refresh_due(base, 900, base + Duration::from_millis(900)));
    }

    #[test]
    fn scan_debounce_blocks_rapid_repeats() {
        let base = Instant::now();
        let remaining = scan_debounce_remaining(Some(base), 700, base + Duration::from_millis(250));
        assert!(remaining.is_some());
        assert!(remaining.unwrap() > Duration::from_millis(400));
    }

    #[test]
    fn scan_debounce_allows_after_window() {
        let base = Instant::now();
        let remaining = scan_debounce_remaining(Some(base), 700, base + Duration::from_millis(701));
        assert!(remaining.is_none());
    }

    #[test]
    fn friendly_wifi_error_simplifies_invalid_passphrase() {
        let err = anyhow::anyhow!("Argument format is invalid");
        assert_eq!(
            friendly_wifi_error("connect/disconnect", &err),
            "Couldn't connect. Check the password and try again."
        );
    }

    #[test]
    fn shorten_banner_message_compacts_and_truncates() {
        let msg = "first line\n\nsecond line with many words ".repeat(10);
        let shortened = shorten_banner_message(&msg);
        assert!(!shortened.contains('\n'));
        assert!(shortened.len() <= 120);
    }
}
