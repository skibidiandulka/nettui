use super::*;
use anyhow::{Context, Result};
use qrcode::{QrCode, render::unicode};
use tokio::process::Command;

impl App {
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

    pub fn open_manual_wifi_auth_prompt(&mut self, ssid: String) {
        self.wifi_auth_prompt = Some(WifiAuthPrompt::manual_passphrase(ssid));
    }

    pub fn open_wifi_auth_prompt_from_agent(&mut self, request: AuthPromptRequest) {
        self.wifi_auth_prompt = Some(WifiAuthPrompt::from_agent_request(request));
    }

    pub fn open_wifi_enterprise_prompt(&mut self, ssid: String) {
        self.wifi_enterprise_prompt = Some(WifiEnterprisePrompt::new(ssid));
    }

    pub fn open_existing_wifi_enterprise_prompt(
        &mut self,
        ssid: String,
        profile: WifiEnterpriseProfile,
        connect_after_save: bool,
    ) {
        self.wifi_enterprise_prompt = Some(WifiEnterprisePrompt::from_existing(
            ssid,
            profile,
            connect_after_save,
        ));
    }

    pub fn close_wifi_auth_prompt(&mut self) {
        if let Some(prompt) = self.wifi_auth_prompt.take() {
            prompt.cancel();
        }
    }

    pub fn dismiss_wifi_auth_prompt(&mut self) {
        self.wifi_auth_prompt = None;
    }

    pub fn close_wifi_enterprise_prompt(&mut self) {
        self.wifi_enterprise_prompt = None;
    }

    pub fn wifi_auth_input_push(&mut self, c: char) {
        let Some(prompt) = self.wifi_auth_prompt.as_mut() else {
            return;
        };
        match prompt.focus {
            WifiAuthPromptField::Primary => prompt.primary_input.push(c),
            WifiAuthPromptField::Secondary => {
                if prompt.has_secondary_field() {
                    prompt.secondary_input.push(c);
                }
            }
        }
    }

    pub fn wifi_auth_input_backspace(&mut self) {
        let Some(prompt) = self.wifi_auth_prompt.as_mut() else {
            return;
        };
        match prompt.focus {
            WifiAuthPromptField::Primary => {
                prompt.primary_input.pop();
            }
            WifiAuthPromptField::Secondary => {
                if prompt.has_secondary_field() {
                    prompt.secondary_input.pop();
                }
            }
        }
    }

    pub fn select_prev_wifi_auth_field(&mut self) {
        if let Some(prompt) = self.wifi_auth_prompt.as_mut()
            && prompt.has_secondary_field()
        {
            prompt.focus = WifiAuthPromptField::Primary;
        }
    }

    pub fn select_next_wifi_auth_field(&mut self) {
        if let Some(prompt) = self.wifi_auth_prompt.as_mut()
            && prompt.has_secondary_field()
        {
            prompt.focus = WifiAuthPromptField::Secondary;
        }
    }

    pub fn toggle_wifi_auth_visibility(&mut self) {
        let Some(prompt) = self.wifi_auth_prompt.as_mut() else {
            return;
        };

        if prompt.has_secondary_field() && prompt.focus == WifiAuthPromptField::Secondary {
            prompt.secondary_visible = !prompt.secondary_visible;
            return;
        }

        if prompt.primary_is_secret() {
            prompt.primary_visible = !prompt.primary_visible;
        } else if prompt.secondary_is_secret() {
            prompt.secondary_visible = !prompt.secondary_visible;
        }
    }

    pub fn select_prev_wifi_enterprise_field(&mut self) {
        if let Some(prompt) = self.wifi_enterprise_prompt.as_mut() {
            prompt.move_prev();
        }
    }

    pub fn select_next_wifi_enterprise_field(&mut self) {
        if let Some(prompt) = self.wifi_enterprise_prompt.as_mut() {
            prompt.move_next();
        }
    }

    pub fn cycle_wifi_enterprise_left(&mut self) {
        if let Some(prompt) = self.wifi_enterprise_prompt.as_mut() {
            prompt.cycle_left();
        }
    }

    pub fn cycle_wifi_enterprise_right(&mut self) {
        if let Some(prompt) = self.wifi_enterprise_prompt.as_mut() {
            prompt.cycle_right();
        }
    }

    pub fn toggle_wifi_enterprise_visibility(&mut self) {
        if let Some(prompt) = self.wifi_enterprise_prompt.as_mut() {
            prompt.toggle_visibility();
        }
    }

    pub fn wifi_enterprise_input_push(&mut self, c: char) {
        let Some(prompt) = self.wifi_enterprise_prompt.as_mut() else {
            return;
        };
        match prompt.focus {
            WifiEnterprisePromptField::Identity => prompt.profile.identity.push(c),
            WifiEnterprisePromptField::ServerDomainMask => {
                prompt.profile.server_domain_mask.push(c);
            }
            WifiEnterprisePromptField::CaCert => prompt.profile.ca_cert.push(c),
            WifiEnterprisePromptField::ClientCert => prompt.profile.client_cert.push(c),
            WifiEnterprisePromptField::ClientKey => prompt.profile.client_key.push(c),
            WifiEnterprisePromptField::KeyPassphrase => {
                prompt.profile.key_passphrase.push(c);
            }
            WifiEnterprisePromptField::Phase2Identity => {
                prompt.profile.phase2_identity.push(c);
            }
            WifiEnterprisePromptField::Phase2Password => {
                prompt.profile.phase2_password.push(c);
            }
            WifiEnterprisePromptField::Password => prompt.profile.password.push(c),
            WifiEnterprisePromptField::Method | WifiEnterprisePromptField::Phase2Method => {}
        }
    }

    pub fn wifi_enterprise_input_backspace(&mut self) {
        let Some(prompt) = self.wifi_enterprise_prompt.as_mut() else {
            return;
        };
        match prompt.focus {
            WifiEnterprisePromptField::Identity => {
                prompt.profile.identity.pop();
            }
            WifiEnterprisePromptField::ServerDomainMask => {
                prompt.profile.server_domain_mask.pop();
            }
            WifiEnterprisePromptField::CaCert => {
                prompt.profile.ca_cert.pop();
            }
            WifiEnterprisePromptField::ClientCert => {
                prompt.profile.client_cert.pop();
            }
            WifiEnterprisePromptField::ClientKey => {
                prompt.profile.client_key.pop();
            }
            WifiEnterprisePromptField::KeyPassphrase => {
                prompt.profile.key_passphrase.pop();
            }
            WifiEnterprisePromptField::Phase2Identity => {
                prompt.profile.phase2_identity.pop();
            }
            WifiEnterprisePromptField::Phase2Password => {
                prompt.profile.phase2_password.pop();
            }
            WifiEnterprisePromptField::Password => {
                prompt.profile.password.pop();
            }
            WifiEnterprisePromptField::Method | WifiEnterprisePromptField::Phase2Method => {}
        }
    }

    pub fn open_wifi_ap_prompt(&mut self) {
        if !self.wifi.has_adapter() {
            self.set_toast(ToastKind::Error, "No Wi-Fi adapter found");
            return;
        }
        self.wifi_ap_prompt_open = true;
        self.wifi_ap_ssid_input.clear();
        self.wifi_ap_passphrase_input.clear();
        self.wifi_ap_passphrase_visible = false;
        self.wifi_ap_prompt_field = WifiApPromptField::Ssid;
    }

    pub fn close_wifi_ap_prompt(&mut self) {
        self.wifi_ap_prompt_open = false;
        self.wifi_ap_ssid_input.clear();
        self.wifi_ap_passphrase_input.clear();
        self.wifi_ap_passphrase_visible = false;
        self.wifi_ap_prompt_field = WifiApPromptField::Ssid;
    }

    pub fn select_prev_wifi_ap_prompt_field(&mut self) {
        self.wifi_ap_prompt_field = match self.wifi_ap_prompt_field {
            WifiApPromptField::Ssid => WifiApPromptField::Ssid,
            WifiApPromptField::Passphrase => WifiApPromptField::Ssid,
        };
    }

    pub fn select_next_wifi_ap_prompt_field(&mut self) {
        self.wifi_ap_prompt_field = match self.wifi_ap_prompt_field {
            WifiApPromptField::Ssid => WifiApPromptField::Passphrase,
            WifiApPromptField::Passphrase => WifiApPromptField::Passphrase,
        };
    }

    pub fn toggle_wifi_ap_passphrase_visibility(&mut self) {
        self.wifi_ap_passphrase_visible = !self.wifi_ap_passphrase_visible;
    }

    pub fn close_wifi_share_popup(&mut self) {
        self.wifi_share_popup = None;
    }

    pub fn wifi_ap_input_push(&mut self, c: char) {
        match self.wifi_ap_prompt_field {
            WifiApPromptField::Ssid => self.wifi_ap_ssid_input.push(c),
            WifiApPromptField::Passphrase => self.wifi_ap_passphrase_input.push(c),
        }
    }

    pub fn wifi_ap_input_backspace(&mut self) {
        match self.wifi_ap_prompt_field {
            WifiApPromptField::Ssid => {
                self.wifi_ap_ssid_input.pop();
            }
            WifiApPromptField::Passphrase => {
                self.wifi_ap_passphrase_input.pop();
            }
        }
    }

    pub async fn submit_hidden_connect(&mut self) {
        if self.wifi_access_point_active() {
            self.set_toast(
                ToastKind::Info,
                "Switch back to station mode before joining a Wi-Fi network",
            );
            return;
        }

        let ssid = self.hidden_ssid_input.trim().to_string();
        if ssid.is_empty() {
            self.set_toast(ToastKind::Error, "SSID cannot be empty");
            return;
        }

        let preferred_iface = self.preferred_station_iface().map(str::to_owned);
        match self
            .wifi_backend
            .connect_hidden(preferred_iface.as_deref(), &ssid)
            .await
        {
            Ok(()) => {
                self.last_action = Some(format!("Connect hidden requested: {ssid}"));
                self.set_toast(ToastKind::Info, format!("Connect hidden requested: {ssid}"));
                self.notify("Wi-Fi", &format!("Connect hidden: {ssid}"));
                self.close_hidden_connect_prompt();
                self.request_refresh();
            }
            Err(e) => {
                let msg = friendly_wifi_error("connect hidden network", &e);
                self.set_toast(ToastKind::Error, msg);
            }
        }
    }

    pub async fn submit_wifi_auth_prompt(&mut self) {
        let Some(mut prompt) = self.wifi_auth_prompt.take() else {
            return;
        };

        if prompt.primary_input.is_empty() {
            self.set_toast(
                ToastKind::Error,
                format!("{} cannot be empty", prompt.primary_label()),
            );
            self.wifi_auth_prompt = Some(prompt);
            return;
        }
        if prompt.has_secondary_field() && prompt.secondary_input.is_empty() {
            let label = prompt.secondary_label().unwrap_or("Password");
            self.set_toast(ToastKind::Error, format!("{label} cannot be empty"));
            self.wifi_auth_prompt = Some(prompt);
            return;
        }

        match prompt.kind {
            WifiAuthPromptKind::ManualPassphrase => {
                if self.wifi_connect_pending {
                    self.set_toast(ToastKind::Info, "Wi-Fi connect already in progress");
                    self.wifi_auth_prompt = Some(prompt);
                    return;
                }

                let ssid = prompt.ssid.clone();
                let passphrase = prompt.primary_input.clone();
                let preferred_iface = self.preferred_station_iface().map(str::to_owned);
                self.last_action = Some(format!("Connecting to {ssid}..."));
                self.wifi_connect_pending = true;
                self.wifi_connect_started_at = Some(Instant::now());
                self.wifi_connect_context = Some(WifiConnectContext {
                    ssid: ssid.clone(),
                    disconnect: false,
                    used_passphrase: true,
                });
                self.wifi_connect_task = Some(tokio::spawn(async move {
                    IwdBackend::new()
                        .connect_with_passphrase(preferred_iface.as_deref(), &ssid, &passphrase)
                        .await
                }));
            }
            WifiAuthPromptKind::AgentPassphrase
            | WifiAuthPromptKind::AgentPrivateKeyPassphrase
            | WifiAuthPromptKind::AgentUserPassword { .. } => {
                if let Some(responder) = prompt.take_responder() {
                    let _ =
                        responder.send(AuthPromptResponse::Secret(prompt.primary_input.clone()));
                }
            }
            WifiAuthPromptKind::AgentUserNameAndPassphrase => {
                if let Some(responder) = prompt.take_responder() {
                    let _ = responder.send(AuthPromptResponse::UserNameAndPassphrase {
                        user_name: prompt.primary_input.clone(),
                        passphrase: prompt.secondary_input.clone(),
                    });
                }
            }
        }
    }

    pub async fn submit_wifi_enterprise_prompt(&mut self) {
        let Some(prompt) = self.wifi_enterprise_prompt.take() else {
            return;
        };

        if self.wifi_connect_pending {
            self.set_toast(ToastKind::Info, "Wi-Fi connect already in progress");
            self.wifi_enterprise_prompt = Some(prompt);
            return;
        }

        if let Err(err) = prompt.profile.validate() {
            self.set_toast(ToastKind::Error, err.to_string());
            self.wifi_enterprise_prompt = Some(prompt);
            return;
        }

        self.set_toast(
            ToastKind::Info,
            "Root access may be required to save enterprise Wi-Fi settings",
        );
        if let Err(err) = self
            .wifi_backend
            .write_enterprise_profile(&prompt.ssid, &prompt.profile)
            .await
        {
            self.set_toast(
                ToastKind::Error,
                friendly_wifi_error("write enterprise profile", &err),
            );
            self.wifi_enterprise_prompt = Some(prompt);
            return;
        }

        let ssid = prompt.ssid.clone();
        if !prompt.connect_after_save {
            self.last_action = Some(format!("Saved enterprise profile for {ssid}"));
            self.set_toast(
                ToastKind::Success,
                format!("Saved enterprise profile for {ssid}"),
            );
            self.notify("Wi-Fi", &format!("Saved enterprise profile for {ssid}"));
            self.request_refresh();
            return;
        }

        let preferred_iface = self.preferred_station_iface().map(str::to_owned);
        self.last_action = Some(format!("Connecting to {ssid}..."));
        self.wifi_connect_pending = true;
        self.wifi_connect_started_at = Some(Instant::now());
        self.wifi_connect_context = Some(WifiConnectContext {
            ssid: ssid.clone(),
            disconnect: false,
            used_passphrase: false,
        });
        self.wifi_connect_task = Some(tokio::spawn(async move {
            IwdBackend::new()
                .connect(preferred_iface.as_deref(), &ssid)
                .await
        }));
    }

    pub async fn wifi_edit_selected_enterprise(&mut self) -> Result<()> {
        if self.wifi_focus != WifiFocus::KnownNetworks {
            self.set_toast(
                ToastKind::Info,
                "Enterprise edit is available in Known Networks",
            );
            return Ok(());
        }

        let Some(net) = self.selected_known_network().cloned() else {
            self.set_toast(ToastKind::Error, "No known network selected");
            return Ok(());
        };

        if !net.is_enterprise() {
            self.set_toast(
                ToastKind::Info,
                "Selected network does not use enterprise Wi-Fi",
            );
            return Ok(());
        }

        self.set_toast(
            ToastKind::Info,
            "Root access may be required to load saved enterprise Wi-Fi settings",
        );
        let ssid = net.ssid.clone();
        let connect_after_save = net.available;

        match self.wifi_backend.read_enterprise_profile(&ssid).await {
            Ok(Some(profile)) => {
                self.open_existing_wifi_enterprise_prompt(ssid, profile, connect_after_save);
            }
            Ok(None) => {
                let mut prompt = WifiEnterprisePrompt::new(ssid);
                prompt.editing_existing = true;
                prompt.connect_after_save = connect_after_save;
                self.wifi_enterprise_prompt = Some(prompt);
                self.set_toast(
                    ToastKind::Info,
                    "No saved enterprise profile was found. Starting a new enterprise setup.",
                );
            }
            Err(err) => {
                self.set_toast(
                    ToastKind::Error,
                    friendly_wifi_error("load enterprise profile", &err),
                );
            }
        }

        Ok(())
    }

    pub async fn wifi_toggle_access_point_mode(&mut self) -> Result<()> {
        if self.wifi_ap_pending {
            self.set_toast(ToastKind::Info, "Access point change already in progress");
            return Ok(());
        }
        if !self.wifi.has_adapter() {
            self.set_toast(ToastKind::Error, "No Wi-Fi adapter found");
            return Ok(());
        }

        if self.wifi_access_point_active() {
            let ssid = self.wifi.access_point_ssid.clone();
            let preferred_iface = self.preferred_access_point_iface().map(str::to_owned);
            self.last_action = Some("Stopping Wi-Fi access point...".to_string());
            self.wifi_ap_pending = true;
            self.wifi_ap_started_at = Some(Instant::now());
            self.wifi_ap_context = Some(WifiApContext {
                ssid,
                stopping: true,
            });
            self.wifi_ap_task = Some(tokio::spawn(async move {
                IwdBackend::new()
                    .stop_access_point(preferred_iface.as_deref())
                    .await
            }));
            return Ok(());
        }

        self.open_wifi_ap_prompt();
        Ok(())
    }

    pub async fn submit_wifi_ap_start(&mut self) {
        let ssid = self.wifi_ap_ssid_input.trim().to_string();
        let passphrase = self.wifi_ap_passphrase_input.clone();

        if ssid.is_empty() {
            self.set_toast(ToastKind::Error, "Access point name cannot be empty");
            return;
        }
        if ssid.contains('/') || ssid.contains('\n') || ssid.contains('\r') {
            self.set_toast(
                ToastKind::Error,
                "Access point name cannot contain / or line breaks",
            );
            return;
        }
        if passphrase.len() < 8 {
            self.set_toast(
                ToastKind::Error,
                "Access point password must be at least 8 characters",
            );
            return;
        }
        if self.wifi_ap_pending {
            self.set_toast(ToastKind::Info, "Access point change already in progress");
            return;
        }

        self.last_action = Some(format!("Starting access point {ssid}..."));
        self.close_wifi_ap_prompt();
        self.wifi_ap_pending = true;
        self.wifi_ap_started_at = Some(Instant::now());
        let preferred_iface = self.preferred_wifi_iface().map(str::to_owned);
        self.wifi_ap_context = Some(WifiApContext {
            ssid: Some(ssid.clone()),
            stopping: false,
        });
        self.wifi_ap_task = Some(tokio::spawn(async move {
            IwdBackend::new()
                .start_access_point(preferred_iface.as_deref(), &ssid, &passphrase)
                .await
        }));
    }

    pub fn notify(&self, title: &str, body: &str) {
        let title = title.to_string();
        let body = body.to_string();
        tokio::spawn(async move {
            let _ = Command::new("notify-send")
                .arg(title)
                .arg(body)
                .arg("-t")
                .arg("2200")
                .output()
                .await;
        });
    }

    pub async fn wifi_scan(&mut self) -> Result<()> {
        if self.wifi_access_point_active() {
            self.set_toast(
                ToastKind::Info,
                "Switch back to station mode before scanning for Wi-Fi networks",
            );
            return Ok(());
        }

        let now = Instant::now();
        if self.wifi_scan_pending {
            self.set_toast(ToastKind::Info, "Wi-Fi scan already running");
            return Ok(());
        }
        if let Some(remaining) =
            scan_debounce_remaining(self.last_scan_request_at, self.config.scan_debounce_ms, now)
        {
            self.set_toast(
                ToastKind::Info,
                format!("Wait {} ms before next scan", remaining.as_millis()),
            );
            return Ok(());
        }

        self.wifi_scan_pending = true;
        self.wifi_scan_started_at = Some(now);
        self.last_scan_request_at = Some(now);
        self.request_refresh();
        self.last_action = Some("Wi-Fi scan requested".to_string());
        let preferred_iface = self.preferred_station_iface().map(str::to_owned);
        self.wifi_scan_task = Some(tokio::spawn(async move {
            IwdBackend::new().scan(preferred_iface.as_deref()).await
        }));
        Ok(())
    }

    pub async fn wifi_connect_or_disconnect(&mut self) -> Result<()> {
        if self.wifi_access_point_active() {
            self.set_toast(
                ToastKind::Info,
                "Stop the access point before joining a Wi-Fi network",
            );
            return Ok(());
        }

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

        if !disconnect && net.is_enterprise() && self.wifi_focus == WifiFocus::NewNetworks {
            self.open_wifi_enterprise_prompt(ssid);
            return Ok(());
        }

        self.wifi_connect_pending = true;
        self.wifi_connect_started_at = Some(Instant::now());
        self.request_refresh();
        self.wifi_connect_context = Some(WifiConnectContext {
            ssid: ssid.clone(),
            disconnect,
            used_passphrase: false,
        });
        let preferred_iface = self.preferred_station_iface().map(str::to_owned);
        self.last_action = Some(if disconnect {
            "Disconnecting Wi-Fi...".to_string()
        } else {
            format!("Connecting to {ssid}...")
        });

        self.wifi_connect_task = Some(tokio::spawn(async move {
            if disconnect {
                IwdBackend::new()
                    .disconnect(preferred_iface.as_deref())
                    .await
            } else {
                IwdBackend::new()
                    .connect(preferred_iface.as_deref(), &ssid)
                    .await
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
                self.notify("Wi-Fi", &format!("Forgot network {}", net.ssid));
                self.request_refresh();
            }
            Err(e) => {
                let msg = friendly_wifi_error("forget known network", &e);
                self.set_toast(ToastKind::Error, msg);
            }
        }
        Ok(())
    }

    pub async fn wifi_share_selected(&mut self) -> Result<()> {
        if self.wifi_focus != WifiFocus::KnownNetworks {
            self.set_toast(ToastKind::Info, "Share is available in Known Networks");
            return Ok(());
        }

        let Some(net) = self.selected_known_network().cloned() else {
            self.set_toast(ToastKind::Error, "No known network selected");
            return Ok(());
        };

        self.set_toast(
            ToastKind::Info,
            "Root access may be required to show saved Wi-Fi credentials",
        );
        let WifiShareCredentials { ssid, passphrase } =
            self.wifi_backend.load_share_credentials(&net.ssid).await?;
        let qr_payload = format!("WIFI:T:WPA;S:{ssid};P:{passphrase};;");
        let qr_text = QrCode::new(qr_payload.as_bytes())
            .context("failed to generate Wi-Fi QR code")?
            .render::<unicode::Dense1x2>()
            .dark_color(unicode::Dense1x2::Dark)
            .light_color(unicode::Dense1x2::Light)
            .build();

        self.wifi_share_popup = Some(WifiSharePopup {
            ssid,
            passphrase,
            qr_text,
        });
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
                self.notify("Wi-Fi", &format!("Autoconnect {}: {}", state, net.ssid));
                self.request_refresh();
            }
            Err(e) => {
                let msg = friendly_wifi_error("toggle autoconnect", &e);
                self.set_toast(ToastKind::Error, msg);
            }
        }
        Ok(())
    }

    pub async fn wifi_toggle_power(&mut self) -> Result<()> {
        let preferred_iface = self.preferred_wifi_iface().map(str::to_owned);
        let powered = self
            .wifi_backend
            .toggle_power(preferred_iface.as_deref())
            .await?;
        self.request_refresh();
        self.last_action = Some(format!(
            "Wi-Fi adapter power {}",
            if powered { "on" } else { "off" }
        ));
        self.set_toast(
            ToastKind::Success,
            format!(
                "Wi-Fi adapter turned {}",
                if powered { "on" } else { "off" }
            ),
        );
        Ok(())
    }

    pub async fn ethernet_renew_dhcp(&mut self) -> Result<()> {
        let iface = self
            .selected_eth_iface()
            .map(|i| i.name.clone())
            .ok_or_else(|| std::io::Error::other("no ethernet interface selected"))?;

        self.set_toast(
            ToastKind::Info,
            format!("Root access may be required to renew DHCP on {iface}"),
        );
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
        self.notify("Ethernet", &format!("DHCP renew requested on {iface}"));
        Ok(())
    }

    pub async fn ethernet_toggle_link(&mut self) -> Result<()> {
        let iface = self
            .selected_eth_iface()
            .cloned()
            .ok_or_else(|| std::io::Error::other("no ethernet interface selected"))?;
        let target_up = !(iface.operstate == "up" || iface.carrier == Some(true));
        let state_word = if target_up { "up" } else { "down" };

        self.set_toast(
            ToastKind::Info,
            format!(
                "Root access may be required to set {} {}",
                iface.name, state_word
            ),
        );
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
        self.notify("Ethernet", &format!("{} link {}", iface.name, state_word));
        Ok(())
    }

    pub fn wifi_scanning_active(&self) -> bool {
        self.wifi_scan_pending
    }

    pub fn wifi_connect_active(&self) -> bool {
        self.wifi_connect_pending
    }

    pub fn wifi_access_point_active(&self) -> bool {
        self.wifi
            .device
            .as_ref()
            .is_some_and(|device| device.mode == "access point")
    }
}
