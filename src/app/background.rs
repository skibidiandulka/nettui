use super::*;

impl App {
    pub fn poll_auth_agent_events(&mut self) {
        loop {
            let event = {
                let Some(receiver) = self.wifi_auth_events.as_mut() else {
                    return;
                };
                receiver.try_recv().ok()
            };

            let Some(event) = event else {
                break;
            };

            match event {
                AuthAgentEvent::Prompt(request) => {
                    let ssid = request.ssid.clone();
                    self.open_wifi_auth_prompt_from_agent(request);
                    self.set_toast(ToastKind::Info, format!("Credentials required for {ssid}"));
                }
                AuthAgentEvent::Cancel { reason } => {
                    self.dismiss_wifi_auth_prompt();
                    self.set_toast(
                        ToastKind::Info,
                        format!("Wi-Fi authentication canceled ({reason})"),
                    );
                }
                AuthAgentEvent::Released => {
                    self.dismiss_wifi_auth_prompt();
                    self.set_toast(
                        ToastKind::Error,
                        "iwd authentication agent was released. Retry the connection.",
                    );
                }
            }
        }
    }

    pub async fn poll_background_tasks(&mut self) {
        let now = Instant::now();
        if self.wifi_scan_pending
            && self.wifi_scan_started_at.is_some_and(|started| {
                now.duration_since(started) > Duration::from_millis(self.config.job_timeout_scan_ms)
            })
        {
            if let Some(handle) = self.wifi_scan_task.take() {
                handle.abort();
            }
            self.wifi_scan_pending = false;
            self.wifi_scan_started_at = None;
            self.set_toast(ToastKind::Error, "Wi-Fi scan timed out");
        }

        if self.wifi_connect_pending
            && self.wifi_connect_started_at.is_some_and(|started| {
                now.duration_since(started)
                    > Duration::from_millis(self.config.job_timeout_connect_ms)
            })
        {
            if let Some(handle) = self.wifi_connect_task.take() {
                handle.abort();
            }
            self.wifi_connect_pending = false;
            self.wifi_connect_started_at = None;
            self.wifi_connect_context = None;
            if !self.has_recent_toast_message("Couldn't talk to iwd.") {
                self.set_toast(ToastKind::Error, "Wi-Fi connect/disconnect timed out");
            }
        }

        if self.wifi_ap_pending
            && self.wifi_ap_started_at.is_some_and(|started| {
                now.duration_since(started)
                    > Duration::from_millis(self.config.job_timeout_connect_ms)
            })
        {
            if let Some(handle) = self.wifi_ap_task.take() {
                handle.abort();
            }
            self.wifi_ap_pending = false;
            self.wifi_ap_started_at = None;
            self.wifi_ap_context = None;
            self.set_toast(ToastKind::Error, "Access point change timed out");
        }

        if let Some(handle) = self.wifi_scan_task.take() {
            if handle.is_finished() {
                self.wifi_scan_pending = false;
                self.wifi_scan_started_at = None;
                match handle.await {
                    Ok(Ok(())) => {
                        self.last_action = Some("Wi-Fi scan completed".to_string());
                        self.set_toast(ToastKind::Success, "Wi-Fi scan completed");
                        self.notify("Wi-Fi", "Scan completed");
                        self.request_refresh();
                    }
                    Ok(Err(e)) => {
                        let msg = friendly_wifi_error("scan", &e);
                        self.set_toast(ToastKind::Error, msg);
                    }
                    Err(e) => {
                        let msg = format!("Scan task failed: {e}");
                        self.set_toast(ToastKind::Error, msg);
                    }
                }
            } else {
                self.wifi_scan_task = Some(handle);
            }
        }

        if let Some(handle) = self.wifi_connect_task.take() {
            if handle.is_finished() {
                self.wifi_connect_pending = false;
                self.wifi_connect_started_at = None;
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
                                self.notify("Wi-Fi", &format!("Disconnected from {}", ctx.ssid));
                            } else {
                                self.last_action = Some(format!("Connected to {}", ctx.ssid));
                                self.set_toast(
                                    ToastKind::Success,
                                    format!("Connected to {}", ctx.ssid),
                                );
                                self.notify("Wi-Fi", &format!("Connected to {}", ctx.ssid));
                            }
                            self.request_refresh();
                        }
                    }
                    Ok(Err(e)) => {
                        let no_agent = is_no_agent_error(&e);
                        let canceled = is_agent_canceled_error(&e);
                        if let Some(ctx) = ctx
                            && no_agent
                            && !ctx.disconnect
                            && !ctx.used_passphrase
                        {
                            self.open_manual_wifi_auth_prompt(ctx.ssid.clone());
                            self.set_toast(
                                ToastKind::Info,
                                format!("Passphrase required for {}", ctx.ssid),
                            );
                        } else if canceled {
                            self.dismiss_wifi_auth_prompt();
                            self.set_toast(ToastKind::Info, "Wi-Fi authentication canceled");
                        } else {
                            let msg = friendly_wifi_error("connect/disconnect", &e);
                            self.set_toast(ToastKind::Error, msg);
                        }
                    }
                    Err(e) => {
                        let msg = format!("Connect task failed: {e}");
                        self.set_toast(ToastKind::Error, msg);
                    }
                }
            } else {
                self.wifi_connect_task = Some(handle);
            }
        }

        if let Some(handle) = self.wifi_ap_task.take() {
            if handle.is_finished() {
                self.wifi_ap_pending = false;
                self.wifi_ap_started_at = None;
                let ctx = self.wifi_ap_context.take();
                match handle.await {
                    Ok(Ok(())) => {
                        if let Some(ctx) = ctx {
                            if ctx.stopping {
                                self.last_action = Some("Stopped Wi-Fi access point".to_string());
                                self.set_toast(ToastKind::Success, "Access point stopped");
                                self.notify("Wi-Fi", "Access point stopped");
                            } else {
                                let ssid = ctx.ssid.unwrap_or_else(|| "hotspot".to_string());
                                self.last_action = Some(format!("Started access point {ssid}"));
                                self.set_toast(
                                    ToastKind::Success,
                                    format!("Access point {ssid} started"),
                                );
                                if !self.wifi_backend.network_configuration_enabled() {
                                    self.set_toast(
                                        ToastKind::Info,
                                        "iwd network configuration is disabled. AP clients may not get DHCP until /etc/iwd/main.conf enables [General] EnableNetworkConfiguration=true.",
                                    );
                                }
                                self.notify("Wi-Fi", &format!("Access point {ssid} started"));
                            }
                            self.request_refresh();
                        }
                    }
                    Ok(Err(e)) => {
                        let msg = friendly_wifi_error("change access point mode", &e);
                        self.set_toast(ToastKind::Error, msg);
                    }
                    Err(e) => {
                        self.set_toast(ToastKind::Error, format!("AP task failed: {e}"));
                    }
                }
            } else {
                self.wifi_ap_task = Some(handle);
            }
        }
    }
}
