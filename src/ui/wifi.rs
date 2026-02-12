use crate::{app::App, domain::common::WifiFocus};
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
            Constraint::Min(6),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(area);

    render_known_networks(app, frame, chunks[0]);
    render_new_networks(app, frame, chunks[1]);
    render_adapter(app, frame, chunks[2]);

    if app.show_wifi_details {
        render_details_popup(app, frame);
    }
}

fn render_known_networks(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.wifi_focus == WifiFocus::KnownNetworks;
    let rows: Vec<Row> = app
        .wifi
        .known_networks
        .iter()
        .map(|n| {
            Row::new(vec![
                Cell::from(if n.connected { "󰤨" } else { "" }),
                Cell::from(n.ssid.clone()),
                Cell::from(n.security.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Min(20),
            Constraint::Length(14),
        ],
    )
    .header(
        Row::new(vec!["", "SSID", "Security"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1),
    )
    .block(section_block(" Known Networks ", focused))
    .row_highlight_style(if focused {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    });

    frame.render_stateful_widget(table, area, &mut app.wifi_known_state);
}

fn render_new_networks(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.wifi_focus == WifiFocus::NewNetworks;
    let rows: Vec<Row> = app
        .wifi
        .new_networks
        .iter()
        .map(|n| {
            Row::new(vec![
                Cell::from(if n.connected { "󰤨" } else { "" }),
                Cell::from(n.ssid.clone()),
                Cell::from(n.security.clone()),
                Cell::from(n.signal.clone()),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Min(20),
            Constraint::Length(14),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["", "SSID", "Security", "Signal"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1),
    )
    .block(section_block(" New Networks ", focused))
    .row_highlight_style(if focused {
        Style::default().bg(Color::DarkGray).fg(Color::White)
    } else {
        Style::default()
    });

    frame.render_stateful_widget(table, area, &mut app.wifi_new_state);
}

fn render_adapter(app: &mut App, frame: &mut Frame, area: Rect) {
    let focused = app.wifi_focus == WifiFocus::Adapter;
    let connected = app
        .wifi
        .connected_ssid
        .clone()
        .unwrap_or_else(|| "-".to_string());

    let rows: Vec<Row> = app
        .wifi
        .ifaces
        .iter()
        .map(|iface| {
            Row::new(vec![
                Cell::from(iface.clone()),
                Cell::from(connected.clone()),
            ])
        })
        .collect();

    let table = Table::new(rows, [Constraint::Length(16), Constraint::Min(20)])
        .header(
            Row::new(vec!["Interface", "Connected SSID"])
                .style(Style::default().fg(Color::Yellow).bold())
                .bottom_margin(1),
        )
        .block(section_block(" Adapter ", focused))
        .row_highlight_style(if focused {
            Style::default().bg(Color::DarkGray).fg(Color::White)
        } else {
            Style::default()
        });

    frame.render_stateful_widget(table, area, &mut app.wifi_adapter_state);
}

fn section_block(title: &str, focused: bool) -> Block<'_> {
    let border = if focused {
        Color::Green
    } else {
        Color::DarkGray
    };
    Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(border))
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
