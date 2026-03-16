use super::popup_layout::centered_rect;
use crate::app::App;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub(super) fn render_info_popups(app: &App, frame: &mut Frame) {
    if app.show_wifi_details {
        render_details_popup(app, frame);
    }
    if app.wifi_share_popup.is_some() {
        render_wifi_share_popup(app, frame);
    }
}

fn render_wifi_share_popup(app: &App, frame: &mut Frame) {
    let Some(share) = &app.wifi_share_popup else {
        return;
    };

    let area = centered_rect(72, 82, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Share Wi-Fi ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(12),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .margin(1)
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("SSID: ").bold(),
            Span::from(share.ssid.as_str()).fg(Color::Cyan),
        ]))
        .alignment(Alignment::Center),
        rows[0],
    );

    frame.render_widget(
        Paragraph::new(share.qr_text.as_str()).alignment(Alignment::Center),
        rows[1],
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("Passphrase: ").bold(),
            Span::from(share.passphrase.as_str())
                .fg(Color::White)
                .bg(Color::DarkGray),
        ]))
        .alignment(Alignment::Center),
        rows[2],
    );
    frame.render_widget(
        Paragraph::new("Scan the QR code or type the password on another device.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::Gray)),
        rows[3],
    );
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("Esc").bold(),
            Span::from(" close"),
        ]))
        .alignment(Alignment::Center),
        rows[4],
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
        lines.push(Line::from(Span::from("IPv6").bold()));
        if details.ipv6.is_empty() {
            lines.push(Line::from("  -"));
        } else {
            for ip in &details.ipv6 {
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
