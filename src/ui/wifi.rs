use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table},
};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(8), Constraint::Length(10)])
        .split(area);

    render_details(app, frame, chunks[0]);
    render_networks(app, frame, chunks[1]);
}

fn render_details(app: &mut App, frame: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Wi-Fi Details ")
        .borders(Borders::ALL)
        .border_type(BorderType::default());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = if !app.wifi.has_adapter() {
        Text::from(vec![
            Line::from("No Wi-Fi adapter found."),
            Line::from(""),
            Line::from("Install/configure iwd and ensure wireless hardware is present."),
        ])
    } else {
        let iface = app
            .wifi
            .ifaces
            .first()
            .cloned()
            .unwrap_or_else(|| "-".into());
        let conn = app
            .wifi
            .connected_ssid
            .clone()
            .unwrap_or_else(|| "Not connected".into());

        let mut lines = vec![
            Line::from(vec![Span::from("Interface: ").bold(), Span::from(iface)]),
            Line::from(vec![
                Span::from("Connected: ").bold(),
                Span::from(conn).fg(Color::Cyan),
            ]),
            Line::from(""),
            Line::from("Actions:"),
            Line::from("  Enter  connect/disconnect selected network"),
            Line::from("  s      scan networks"),
        ];

        if let Some(msg) = &app.last_action {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::from("Last: ").bold(),
                Span::from(msg.clone()).fg(Color::Cyan),
            ]));
        }

        Text::from(lines)
    };

    let p = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(p, inner);
}

fn render_networks(app: &mut App, frame: &mut Frame, area: Rect) {
    let rows: Vec<Row> = app
        .wifi
        .networks
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
            Constraint::Length(12),
            Constraint::Length(8),
        ],
    )
    .header(
        Row::new(vec!["", "SSID", "Security", "Signal"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(" Networks ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_type(BorderType::Thick),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_stateful_widget(table, area, &mut app.wifi_state);
}
