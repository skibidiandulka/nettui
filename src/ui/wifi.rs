// Copyright (C) 2026 skibidiandulka
// Clean-room implementation inspired by Impala UX by pythops.

use crate::{app::App, domain::common::WifiFocus, domain::wifi::WifiDeviceInfo};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table},
};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(7),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(area);

    render_known_networks(app, frame, chunks[0]);
    render_new_networks(app, frame, chunks[1]);
    render_device(app, frame, chunks[2]);

    if app.show_wifi_details {
        render_details_popup(app, frame);
    }
    if app.wifi_passphrase_prompt_ssid.is_some() {
        render_wifi_passphrase_popup(app, frame);
    }
    if app.hidden_connect_prompt {
        render_hidden_connect_popup(app, frame);
    }
}

fn render_known_networks(app: &mut App, frame: &mut Frame, area: Rect) {
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
                .dark_gray(),
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

fn render_new_networks(app: &mut App, frame: &mut Frame, area: Rect) {
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
                .dark_gray(),
            );
        }
    }

    if rows.is_empty() {
        rows.push(Row::new(vec![
            Cell::from("- no new networks -").dark_gray(),
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

fn render_device(app: &mut App, frame: &mut Frame, area: Rect) {
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

fn section_block(title: &str, focused: bool) -> Block<'_> {
    let border = if focused { Color::Green } else { Color::White };
    let border_type = if focused {
        BorderType::Thick
    } else {
        BorderType::Plain
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(Style::default().fg(border))
}

fn render_hidden_connect_popup(app: &App, frame: &mut Frame) {
    let area = centered_rect(58, 28, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Connect Hidden Network ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let content = vec![
        Line::from("Enter hidden SSID"),
        Line::from(""),
        Line::from(vec![
            Span::from("SSID: ").bold(),
            Span::from(app.hidden_ssid_input.clone()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::from("Enter").bold(),
            Span::from(" connect"),
            Span::from(" | "),
            Span::from("Esc").bold(),
            Span::from(" cancel"),
        ]),
    ];
    frame.render_widget(Paragraph::new(content), inner);
}

fn render_wifi_passphrase_popup(app: &App, frame: &mut Frame) {
    let Some(ssid) = app.wifi_passphrase_prompt_ssid.clone() else {
        return;
    };

    let area = centered_rect(62, 32, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Wi-Fi Passphrase ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let masked = "*".repeat(app.wifi_passphrase_input.chars().count());
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(inner);

    let content = vec![Line::from(vec![
        Span::from("SSID: ").bold(),
        Span::from(ssid).fg(Color::Cyan),
    ])];
    frame.render_widget(Paragraph::new(content), chunks[0]);

    frame.render_widget(
        Paragraph::new(Line::from("Passphrase:").style(Style::default().bold())),
        chunks[1],
    );

    let field_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);
    let field_inner = field_block.inner(chunks[2]);
    frame.render_widget(field_block, chunks[2]);
    frame.render_widget(Paragraph::new(Line::from(Span::from(masked))), field_inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("↵").bold(),
            Span::from(" connect"),
            Span::from(" | "),
            Span::from("Esc").bold(),
            Span::from(" cancel"),
        ])),
        chunks[4],
    );
}

fn render_details_popup(app: &App, frame: &mut Frame) {
    let area = centered_rect(78, 70, frame.area());
    frame.render_widget(Clear, area);

    let title = app
        .wifi
        .ifaces
        .first()
        .map(|i| format!(" Wi-Fi Details ({i}) "))
        .unwrap_or_else(|| " Wi-Fi Details ".to_string());

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = if let Some(details) = &app.wifi_iface_details {
        let mut lines = vec![
            Line::from(vec![
                Span::from("Connected SSID: ").bold(),
                Span::from(
                    app.wifi
                        .connected_ssid
                        .clone()
                        .unwrap_or_else(|| "Not connected".to_string()),
                )
                .fg(Color::Cyan),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::from("State: ").bold(),
                Span::from(details.operstate.clone()),
            ]),
            Line::from(vec![
                Span::from("Carrier: ").bold(),
                Span::from(
                    details
                        .carrier
                        .map(|c| if c { "1" } else { "0" })
                        .unwrap_or("?"),
                ),
            ]),
            Line::from(vec![
                Span::from("Speed: ").bold(),
                Span::from(
                    details
                        .speed_mbps
                        .map(|s| format!("{s} Mb/s"))
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ]),
            Line::from(vec![
                Span::from("MAC: ").bold(),
                Span::from(details.mac.clone().unwrap_or_else(|| "-".to_string())),
            ]),
            Line::from(vec![
                Span::from("Gateway v4: ").bold(),
                Span::from(
                    details
                        .gateway_v4
                        .clone()
                        .unwrap_or_else(|| "-".to_string()),
                ),
            ]),
            Line::from(""),
            Line::from(Span::from("IPv4").bold()),
        ];

        if details.ipv4.is_empty() {
            lines.push(Line::from("  -"));
        } else {
            for ip in &details.ipv4 {
                lines.push(Line::from(format!("  {ip}")));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::from("i").bold(),
            Span::from(" close details"),
        ]));
        lines
    } else {
        vec![
            Line::from("No Wi-Fi interface details available."),
            Line::from(""),
            Line::from("Make sure a physical Wi-Fi adapter is present."),
            Line::from(""),
            Line::from(vec![Span::from("i").bold(), Span::from(" close details")]),
        ]
    };

    let paragraph = Paragraph::new(lines).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(paragraph, inner);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
