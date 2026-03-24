use super::styles::{secondary_row_style, secondary_text_style, section_block};
use crate::{app::App, domain::common::WifiFocus, domain::wifi::WifiDeviceInfo};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    widgets::{Cell, Row, Table},
};

pub(super) fn render_known_networks(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.wifi_access_point_active() {
        render_access_point_status(app, frame, area);
        return;
    }

    let focused = app.wifi_focus == WifiFocus::KnownNetworks;
    let title = if app.wifi_connect_active() {
        " Known Networks (Connecting) ".to_string()
    } else {
        " Known Networks ".to_string()
    };
    let mut rows: Vec<Row> = app
        .wifi
        .known_networks
        .iter()
        .map(|n| {
            Row::new(vec![
                Cell::from(if n.connected { "󰖩" } else { "" }),
                Cell::from(n.ssid.clone()),
                Cell::from(n.security.clone()),
                Cell::from(
                    n.hidden
                        .map(|v| if v { "Yes" } else { "No" })
                        .unwrap_or("-"),
                ),
                Cell::from(
                    n.autoconnect
                        .map(|v| if v { "Yes" } else { "No" })
                        .unwrap_or("-"),
                ),
                Cell::from(n.signal.clone()),
            ])
        })
        .collect();

    if app.show_unavailable_known_networks {
        for n in &app.wifi.unavailable_known_networks {
            rows.push(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(n.ssid.clone()),
                    Cell::from(n.security.clone()),
                    Cell::from(
                        n.hidden
                            .map(|v| if v { "Yes" } else { "No" })
                            .unwrap_or("-"),
                    ),
                    Cell::from(
                        n.autoconnect
                            .map(|v| if v { "Yes" } else { "No" })
                            .unwrap_or("-"),
                    ),
                    Cell::from("-"),
                ])
                .style(secondary_row_style()),
            );
        }
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(28),
            Constraint::Length(10),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec![
            "",
            "Name",
            "Security",
            "Hidden",
            "Auto Connect",
            "Signal",
        ])
        .style(Style::default().fg(Color::Yellow).bold())
        .bottom_margin(1),
    )
    .block(section_block(&title, focused))
    .row_highlight_style(if focused {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    });

    frame.render_stateful_widget(table, area, &mut app.wifi_known_state);
}

pub(super) fn render_new_networks(app: &mut App, frame: &mut Frame, area: Rect) {
    if app.wifi_access_point_active() {
        render_access_point_clients(app, frame, area);
        return;
    }

    let focused = app.wifi_focus == WifiFocus::NewNetworks;
    let title = if app.wifi_scanning_active() {
        " New Networks (Scanning) ".to_string()
    } else if app.wifi_connect_active() {
        " New Networks (Connecting) ".to_string()
    } else {
        " New Networks ".to_string()
    };
    let mut rows: Vec<Row> = app
        .wifi
        .new_networks
        .iter()
        .map(|n| {
            Row::new(vec![
                Cell::from(n.ssid.clone()),
                Cell::from(n.security.clone()),
                Cell::from(n.signal.clone()),
            ])
        })
        .collect();

    if app.show_hidden_networks {
        for n in &app.wifi.hidden_networks {
            rows.push(
                Row::new(vec![
                    Cell::from(n.ssid.clone()),
                    Cell::from(n.security.clone()),
                    Cell::from(n.signal.clone()),
                ])
                .style(secondary_row_style()),
            );
        }
    }

    if rows.is_empty() {
        rows.push(Row::new(vec![
            Cell::from("- no new networks -").style(secondary_text_style()),
            Cell::from(""),
            Cell::from(""),
        ]));
    }

    let table = Table::new(
        rows,
        [
            Constraint::Length(34),
            Constraint::Length(12),
            Constraint::Length(10),
        ],
    )
    .header(
        Row::new(vec!["Name", "Security", "Signal"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1),
    )
    .block(section_block(&title, focused))
    .row_highlight_style(if focused {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    });

    frame.render_stateful_widget(table, area, &mut app.wifi_new_state);
}

fn render_access_point_status(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.wifi_focus == WifiFocus::KnownNetworks;
    let title = if app.wifi_ap_pending {
        " Access Point (Updating) "
    } else {
        " Access Point "
    };
    let rows = vec![Row::new(vec![
        Cell::from(
            app.wifi
                .access_point_ssid
                .clone()
                .unwrap_or_else(|| "-".to_string()),
        ),
        Cell::from(
            app.wifi
                .access_point_iface
                .clone()
                .or_else(|| app.wifi.device.as_ref().map(|device| device.iface.clone()))
                .unwrap_or_else(|| "-".to_string()),
        ),
        Cell::from(
            app.wifi
                .device
                .as_ref()
                .map(|device| device.state.clone())
                .unwrap_or_else(|| "-".to_string()),
        ),
        Cell::from(
            app.wifi
                .device
                .as_ref()
                .map(|device| device.frequency.clone())
                .unwrap_or_else(|| "-".to_string()),
        ),
        Cell::from(
            app.wifi
                .device
                .as_ref()
                .map(|device| device.security.clone())
                .unwrap_or_else(|| "-".to_string()),
        ),
    ])];

    let table = Table::new(
        rows,
        [
            Constraint::Length(24),
            Constraint::Length(12),
            Constraint::Length(12),
            Constraint::Length(14),
            Constraint::Length(14),
        ],
    )
    .header(
        Row::new(vec!["SSID", "Interface", "State", "Frequency", "Security"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1),
    )
    .block(section_block(title, focused))
    .row_highlight_style(if focused {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    });

    frame.render_stateful_widget(table, area, &mut app.wifi_known_state);
}

fn render_access_point_clients(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.wifi_focus == WifiFocus::NewNetworks;
    let title = " Connected Devices ";
    let mut rows: Vec<Row> = app
        .wifi
        .access_point_clients
        .iter()
        .map(|client| Row::new(vec![Cell::from(client.clone())]))
        .collect();

    if rows.is_empty() {
        rows.push(Row::new(vec![
            Cell::from("- no connected devices -").style(secondary_text_style()),
        ]));
        if app.wifi_ap_needs_network_configuration() {
            rows.push(Row::new(vec![
                Cell::from("Enable iwd network configuration for DHCP support.")
                    .style(secondary_text_style()),
            ]));
        }
    }

    let table = Table::new(rows, [Constraint::Min(20)])
        .header(
            Row::new(vec!["Address"])
                .style(Style::default().fg(Color::Yellow).bold())
                .bottom_margin(1),
        )
        .block(section_block(title, focused))
        .row_highlight_style(if focused {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        });

    frame.render_stateful_widget(table, area, &mut app.wifi_new_state);
}

pub(super) fn render_device(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.wifi_focus == WifiFocus::Adapter;
    let dev = app.wifi.device.clone().unwrap_or_else(|| WifiDeviceInfo {
        iface: app
            .wifi
            .ifaces
            .first()
            .cloned()
            .unwrap_or_else(|| "-".to_string()),
        mode: "station".to_string(),
        powered: "-".to_string(),
        state: "-".to_string(),
        scanning: "-".to_string(),
        frequency: "-".to_string(),
        security: "-".to_string(),
    });

    let rows = vec![Row::new(vec![
        Cell::from(dev.iface),
        Cell::from(dev.mode),
        Cell::from(dev.powered),
        Cell::from(dev.state),
        Cell::from(dev.scanning),
        Cell::from(dev.frequency),
        Cell::from(dev.security),
    ])];

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(14),
            Constraint::Percentage(11),
            Constraint::Percentage(11),
            Constraint::Percentage(14),
            Constraint::Percentage(14),
            Constraint::Percentage(16),
            Constraint::Percentage(20),
        ],
    )
    .header(
        Row::new(vec![
            "Name",
            "Mode",
            "Powered",
            "State",
            "Scanning",
            "Frequency",
            "Security",
        ])
        .style(Style::default().fg(Color::Yellow).bold())
        .bottom_margin(1),
    )
    .column_spacing(1)
    .block(section_block(" Device ", focused))
    .row_highlight_style(if focused {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    });

    frame.render_stateful_widget(table, area, &mut app.wifi_adapter_state);
}
