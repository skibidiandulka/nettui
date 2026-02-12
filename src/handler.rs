use crate::{app::App, domain::common::ActiveTab};
use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> Result<()> {
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
            if let Err(e) = app.wifi_scan().await {
                app.last_error = Some(e.to_string());
            } else {
                app.refresh_current().await;
            }
        }

        KeyCode::Enter if app.active_tab == ActiveTab::Wifi => {
            app.clear_error();
            if let Err(e) = app.wifi_connect_or_disconnect().await {
                app.last_error = Some(e.to_string());
            } else {
                app.refresh_current().await;
            }
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
