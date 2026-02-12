use crate::{
    app::App,
    domain::common::{ActiveTab, WifiFocus},
};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
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
        KeyCode::Char('q') => app.quit(),
        KeyCode::Esc if app.config.esc_quit => app.quit(),
        KeyCode::Char('c' | 'C') if key_event.modifiers == KeyModifiers::CONTROL => app.quit(),

        KeyCode::Right | KeyCode::Char('l') => app.switch_transport_next(),
        KeyCode::Left | KeyCode::Char('h') => app.switch_transport_prev(),
        KeyCode::Tab => app.switch_focus_next(),
        KeyCode::BackTab => app.switch_focus_prev(),

        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_prev(),

        KeyCode::Char('r') => {
            app.clear_error();
            app.refresh_current().await;
        }

        KeyCode::Char('s') if app.active_tab == ActiveTab::Wifi => {
            app.clear_error();
            app.wifi_scan().await?;
            app.refresh_current().await;
        }

        KeyCode::Char('a') if app.active_tab == ActiveTab::Wifi => match app.wifi_focus {
            WifiFocus::KnownNetworks => app.toggle_known_show_all(),
            WifiFocus::NewNetworks => app.toggle_new_show_all(),
            WifiFocus::Adapter => {}
        },

        KeyCode::Char('d')
            if app.active_tab == ActiveTab::Wifi && app.wifi_focus == WifiFocus::KnownNetworks =>
        {
            app.clear_error();
            app.wifi_forget_selected().await?;
            app.refresh_current().await;
        }

        KeyCode::Char('t')
            if app.active_tab == ActiveTab::Wifi && app.wifi_focus == WifiFocus::KnownNetworks =>
        {
            app.clear_error();
            app.wifi_toggle_autoconnect_selected().await?;
            app.refresh_current().await;
        }

        KeyCode::Char('n')
            if app.active_tab == ActiveTab::Wifi && app.wifi_focus == WifiFocus::NewNetworks =>
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
            app.refresh_current().await;
        }

        KeyCode::Char('i') if app.active_tab == ActiveTab::Wifi => {
            app.toggle_wifi_details();
        }

        KeyCode::Char('n') if app.active_tab == ActiveTab::Ethernet => {
            app.clear_error();
            if let Err(e) = app.ethernet_renew_dhcp().await {
                app.last_error = Some(e.to_string());
            } else {
                app.refresh_current().await;
            }
        }

        _ => {}
    }

    Ok(())
}
