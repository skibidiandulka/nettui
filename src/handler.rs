use crate::{
    app::App,
    domain::common::{ActiveTab, ToastKind, WifiFocus},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if app.wifi_share_popup.is_some() {
        match key_event.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char(' ') => app.close_wifi_share_popup(),
            _ => {}
        }
        return Ok(());
    }

    if app.wifi_ap_prompt_open {
        match key_event.code {
            KeyCode::Esc => app.close_wifi_ap_prompt(),
            KeyCode::Tab | KeyCode::BackTab => app.toggle_wifi_ap_passphrase_visibility(),
            KeyCode::Enter => app.submit_wifi_ap_start().await,
            KeyCode::Up => app.select_prev_wifi_ap_prompt_field(),
            KeyCode::Down => app.select_next_wifi_ap_prompt_field(),
            KeyCode::Backspace => app.wifi_ap_input_backspace(),
            KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.wifi_ap_input_push(c)
            }
            _ => {}
        }
        return Ok(());
    }

    if app.wifi_enterprise_prompt.is_some() {
        match key_event.code {
            KeyCode::Esc => app.close_wifi_enterprise_prompt(),
            KeyCode::Tab | KeyCode::BackTab => app.toggle_wifi_enterprise_visibility(),
            KeyCode::Enter => app.submit_wifi_enterprise_prompt().await,
            KeyCode::Up => app.select_prev_wifi_enterprise_field(),
            KeyCode::Down => app.select_next_wifi_enterprise_field(),
            KeyCode::Left => app.cycle_wifi_enterprise_left(),
            KeyCode::Right => app.cycle_wifi_enterprise_right(),
            KeyCode::Backspace => app.wifi_enterprise_input_backspace(),
            KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.wifi_enterprise_input_push(c)
            }
            _ => {}
        }
        return Ok(());
    }

    if app.wifi_auth_prompt.is_some() {
        match key_event.code {
            KeyCode::Esc => app.close_wifi_auth_prompt(),
            KeyCode::Tab | KeyCode::BackTab => app.toggle_wifi_auth_visibility(),
            KeyCode::Enter => app.submit_wifi_auth_prompt().await,
            KeyCode::Up => app.select_prev_wifi_auth_field(),
            KeyCode::Down => app.select_next_wifi_auth_field(),
            KeyCode::Backspace => app.wifi_auth_input_backspace(),
            KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.wifi_auth_input_push(c)
            }
            _ => {}
        }
        return Ok(());
    }

    if app.hidden_connect_prompt {
        match key_event.code {
            KeyCode::Esc => app.close_hidden_connect_prompt(),
            KeyCode::Enter => app.submit_hidden_connect().await,
            KeyCode::Backspace => app.hidden_input_backspace(),
            KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.hidden_input_push(c)
            }
            _ => {}
        }
        return Ok(());
    }

    match key_event.code {
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&app.keybinds.quit) => app.quit(),
        KeyCode::Esc if app.config.esc_quit => app.quit(),
        KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => app.quit(),

        KeyCode::Right => app.switch_transport_next(),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&app.keybinds.next_tab) => {
            app.switch_transport_next()
        }
        KeyCode::Left => app.switch_transport_prev(),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&app.keybinds.prev_tab) => {
            app.switch_transport_prev()
        }
        KeyCode::Tab => app.switch_focus_next(),
        KeyCode::BackTab => app.switch_focus_prev(),

        KeyCode::Down => app.select_next(),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&app.keybinds.down) => app.select_next(),
        KeyCode::Up => app.select_prev(),
        KeyCode::Char(c) if c.eq_ignore_ascii_case(&app.keybinds.up) => app.select_prev(),

        KeyCode::Char(c) if c.eq_ignore_ascii_case(&app.keybinds.refresh) => {
            if app.block_if_busy("refreshing") {
                return Ok(());
            }
            app.clear_error();
            app.refresh_current().await;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_scan) =>
        {
            if app.block_if_busy("starting another Wi-Fi scan") {
                return Ok(());
            }
            app.clear_error();
            app.wifi_scan().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_show_all) =>
        {
            match app.wifi_focus {
                WifiFocus::KnownNetworks => app.toggle_known_show_all(),
                WifiFocus::NewNetworks => app.toggle_new_show_all(),
                WifiFocus::Adapter => {}
            }
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::KnownNetworks
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_forget) =>
        {
            if app.block_if_busy("forgetting a network") {
                return Ok(());
            }
            app.clear_error();
            app.wifi_forget_selected().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::KnownNetworks
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_share) =>
        {
            if app.block_if_busy("sharing a network") {
                return Ok(());
            }
            app.clear_error();
            if let Err(e) = app.wifi_share_selected().await {
                app.set_toast(ToastKind::Error, e.to_string());
            }
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::KnownNetworks
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_autoconnect) =>
        {
            if app.block_if_busy("changing autoconnect") {
                return Ok(());
            }
            app.clear_error();
            app.wifi_toggle_autoconnect_selected().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::NewNetworks
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_hidden) =>
        {
            if app.block_if_busy("opening hidden network connect") {
                return Ok(());
            }
            app.open_hidden_connect_prompt();
        }

        KeyCode::Enter | KeyCode::Char(' ')
            if app.active_tab == ActiveTab::Wifi
                && matches!(
                    app.wifi_focus,
                    WifiFocus::KnownNetworks | WifiFocus::NewNetworks
                ) =>
        {
            if app.block_if_busy("changing Wi-Fi connection") {
                return Ok(());
            }
            app.clear_error();
            app.wifi_connect_or_disconnect().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_details) =>
        {
            app.toggle_wifi_details();
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::Adapter
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_power) =>
        {
            if app.block_if_busy("changing Wi-Fi power") {
                return Ok(());
            }
            app.clear_error();
            if let Err(e) = app.wifi_toggle_power().await {
                app.set_toast(ToastKind::Error, e.to_string());
            }
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_access_point) =>
        {
            if app.block_if_busy("changing access point mode") {
                return Ok(());
            }
            app.clear_error();
            app.wifi_toggle_access_point_mode().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Ethernet
                && c.eq_ignore_ascii_case(&app.keybinds.ethernet_renew) =>
        {
            if app.block_if_busy("running Ethernet renew") {
                return Ok(());
            }
            app.clear_error();
            if let Err(e) = app.ethernet_renew_dhcp().await {
                let msg = e.to_string();
                app.set_toast(ToastKind::Error, msg);
            }
        }

        KeyCode::Enter if app.active_tab == ActiveTab::Ethernet => {
            if app.block_if_busy("changing Ethernet link state") {
                return Ok(());
            }
            app.clear_error();
            if let Err(e) = app.ethernet_toggle_link().await {
                let msg = e.to_string();
                app.set_toast(ToastKind::Error, msg);
            }
        }

        _ => {}
    }

    Ok(())
}
