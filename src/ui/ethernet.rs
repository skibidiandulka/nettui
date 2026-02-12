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
    render_ifaces(app, frame, chunks[1]);
}

fn render_details(app: &mut App, frame: &mut Frame, area: Rect) {
    let title = if let Some(d) = app.selected_eth_iface() {
        format!(" Ethernet Details ({}) ", d.name)
    } else {
        " Ethernet Details ".to_string()
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::default());

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let text = if let Some(d) = app.selected_eth_iface() {
        let mut lines = Vec::new();
        lines.push(Line::from(vec![
            Span::from("State: ").bold(),
            Span::from(d.operstate.clone()),
        ]));
        lines.push(Line::from(vec![
            Span::from("Carrier: ").bold(),
            Span::from(
                d.carrier
                    .map(|c| if c { "1" } else { "0" })
                    .unwrap_or("?")
                    .to_string(),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::from("Speed: ").bold(),
            Span::from(
                d.speed_mbps
                    .map(|s| format!("{s} Mb/s"))
                    .unwrap_or_else(|| "-".into()),
            ),
        ]));
        lines.push(Line::from(vec![
            Span::from("MAC: ").bold(),
            Span::from(d.mac.clone().unwrap_or_else(|| "-".into())),
        ]));
        lines.push(Line::from(vec![
            Span::from("Gateway v4: ").bold(),
            Span::from(d.gateway_v4.clone().unwrap_or_else(|| "-".into())),
        ]));

        lines.push(Line::from(""));
        lines.push(Line::from("IPv4:"));
        if d.ipv4.is_empty() {
            lines.push(Line::from("  -"));
        } else {
            for ip in &d.ipv4 {
                lines.push(Line::from(format!("  {ip}")));
            }
        }

        if let Some(msg) = &app.last_action {
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::from("Last: ").bold(),
                Span::from(msg.clone()).fg(Color::Cyan),
            ]));
        }

        Text::from(lines)
    } else {
        Text::from(vec![
            Line::from("No Ethernet adapter found."),
            Line::from(""),
            Line::from("This panel lists physical non-wifi interfaces."),
        ])
    };

    let p = Paragraph::new(text).wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(p, inner);
}

fn render_ifaces(app: &mut App, frame: &mut Frame, area: Rect) {
    let rows: Vec<Row> = app
        .ethernet
        .ifaces
        .iter()
        .map(|d| {
            let carrier = d.carrier.map(|c| if c { "1" } else { "0" }).unwrap_or("?");
            let speed = d
                .speed_mbps
                .map(|s| s.to_string())
                .unwrap_or_else(|| "-".to_string());
            let active = if d.is_active() { "󰀂" } else { "" };

            Row::new(vec![
                Cell::from(active),
                Cell::from(d.name.clone()),
                Cell::from(d.operstate.clone()),
                Cell::from(carrier),
                Cell::from(speed),
                Cell::from(d.ipv4.first().cloned().unwrap_or_else(|| "-".into())),
            ])
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),
            Constraint::Length(10),
            Constraint::Length(9),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Min(12),
        ],
    )
    .header(
        Row::new(vec!["", "Iface", "State", "Carrier", "Speed", "IPv4"])
            .style(Style::default().fg(Color::Yellow).bold())
            .bottom_margin(1),
    )
    .block(
        Block::default()
            .title(" Interfaces ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green))
            .border_type(BorderType::Thick),
    )
    .row_highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White));

    frame.render_stateful_widget(table, area, &mut app.ethernet_state);
}
