use crate::{
    app::App,
    domain::common::{ActiveTab, WifiFocus},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
    if app.wifi_passphrase_prompt_ssid.is_some() {
        match key_event.code {
            KeyCode::Esc => app.close_wifi_passphrase_prompt(),
            KeyCode::Enter => app.submit_wifi_passphrase_connect().await,
            KeyCode::Backspace => app.passphrase_input_backspace(),
            KeyCode::Char(c) if !key_event.modifiers.contains(KeyModifiers::CONTROL) => {
                app.passphrase_input_push(c)
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
            app.clear_error();
            app.refresh_current().await;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_scan) =>
        {
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
            app.clear_error();
            app.wifi_forget_selected().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::KnownNetworks
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_autoconnect) =>
        {
            app.clear_error();
            app.wifi_toggle_autoconnect_selected().await?;
        }

        KeyCode::Char(c)
            if app.active_tab == ActiveTab::Wifi
                && app.wifi_focus == WifiFocus::NewNetworks
                && c.eq_ignore_ascii_case(&app.keybinds.wifi_hidden) =>
        {
            app.open_hidden_connect_prompt();
        }

        KeyCode::Enter | KeyCode::Char(' ')
            if app.active_tab == ActiveTab::Wifi
                && matches!(
                    app.wifi_focus,
                    WifiFocus::KnownNetworks | WifiFocus::NewNetworks
                ) =>
        {
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
            if app.active_tab == ActiveTab::Ethernet
                && c.eq_ignore_ascii_case(&app.keybinds.ethernet_renew) =>
        {
            app.clear_error();
            if let Err(e) = app.ethernet_renew_dhcp().await {
                app.last_error = Some(e.to_string());
            }
        }

        KeyCode::Enter if app.active_tab == ActiveTab::Ethernet => {
            app.clear_error();
            if let Err(e) = app.ethernet_toggle_link().await {
                app.last_error = Some(e.to_string());
            }
        }

        _ => {}
    }

    Ok(())
}
