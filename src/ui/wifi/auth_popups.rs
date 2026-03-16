use super::popup_layout::centered_rect;
use crate::app::{App, WifiApPromptField};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};

pub(super) fn render_auth_popups(app: &App, frame: &mut Frame) {
    if app.wifi_ap_prompt_open {
        render_wifi_ap_popup(app, frame);
    }
    if app.wifi_passphrase_prompt_ssid.is_some() {
        render_wifi_passphrase_popup(app, frame);
    }
    if app.hidden_connect_prompt {
        render_hidden_connect_popup(app, frame);
    }
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

    let area = centered_rect(62, 38, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Wi-Fi Passphrase ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let passphrase = if app.wifi_passphrase_visible {
        app.wifi_passphrase_input.clone()
    } else {
        "*".repeat(app.wifi_passphrase_input.chars().count())
    };
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(8),
            Constraint::Fill(1),
        ])
        .split(inner);
    let content_area = outer[1];
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
        ])
        .split(content_area);
    let label_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(chunks[0])[1];
    let field_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(chunks[2])[1];

    let content = vec![Line::from(vec![
        Span::from("SSID: ").bold(),
        Span::from(ssid).fg(Color::Cyan),
    ])];
    frame.render_widget(Paragraph::new(content), label_area);

    frame.render_widget(
        Paragraph::new(Line::from("Passphrase:").style(Style::default().bold())),
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(42),
                Constraint::Fill(1),
            ])
            .split(chunks[1])[1],
    );

    let field_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);
    let field_inner = field_block.inner(field_area);
    frame.render_widget(field_block, field_area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::from(passphrase))).alignment(Alignment::Center),
        field_inner,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("⇥").bold(),
            Span::from(" show/hide"),
            Span::from(" | "),
            Span::from("↵").bold(),
            Span::from(" connect"),
            Span::from(" | "),
            Span::from("Esc").bold(),
            Span::from(" cancel"),
        ]))
        .alignment(Alignment::Center),
        chunks[3],
    );
}

fn render_wifi_ap_popup(app: &App, frame: &mut Frame) {
    let area = centered_rect(70, 54, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Start Access Point ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(15),
            Constraint::Fill(1),
        ])
        .split(inner);
    let content_area = outer[1];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Length(4),
        ])
        .split(content_area);

    let ssid_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(rows[1])[1];
    let passphrase_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(rows[3])[1];

    frame.render_widget(
        Paragraph::new(Line::from("SSID:").style(Style::default().bold())),
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(42),
                Constraint::Fill(1),
            ])
            .split(rows[0])[1],
    );
    frame.render_widget(
        Paragraph::new(Line::from("Password:").style(Style::default().bold())),
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(42),
                Constraint::Fill(1),
            ])
            .split(rows[2])[1],
    );

    let ssid_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if app.wifi_ap_prompt_field == WifiApPromptField::Ssid {
                Color::Cyan
            } else {
                Color::Gray
            }),
        )
        .border_type(BorderType::Rounded);
    let ssid_inner = ssid_block.inner(ssid_area);
    frame.render_widget(ssid_block, ssid_area);
    frame.render_widget(
        Paragraph::new(Line::from(app.wifi_ap_ssid_input.as_str())),
        ssid_inner,
    );

    let passphrase_value = if app.wifi_ap_passphrase_visible {
        app.wifi_ap_passphrase_input.clone()
    } else {
        "*".repeat(app.wifi_ap_passphrase_input.chars().count())
    };
    let passphrase_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(
            if app.wifi_ap_prompt_field == WifiApPromptField::Passphrase {
                Color::Cyan
            } else {
                Color::Gray
            },
        ))
        .border_type(BorderType::Rounded);
    let passphrase_inner = passphrase_block.inner(passphrase_area);
    frame.render_widget(passphrase_block, passphrase_area);
    frame.render_widget(
        Paragraph::new(Line::from(passphrase_value)),
        passphrase_inner,
    );

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("↑/↓").bold(),
            Span::from(" field"),
            Span::from(" | "),
            Span::from("⇥").bold(),
            Span::from(" show/hide"),
            Span::from(" | "),
            Span::from("↵").bold(),
            Span::from(" start AP"),
            Span::from(" | "),
            Span::from("Esc").bold(),
            Span::from(" cancel"),
        ]))
        .alignment(Alignment::Center),
        rows[4],
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from(""),
            Line::from("Hotspot support depends on the Wi-Fi adapter and driver.")
                .style(Style::default().fg(Color::Yellow)),
            Line::from(
                "Some adapters can scan and connect normally but still fail in access point mode.",
            )
            .style(Style::default().fg(Color::Gray)),
            Line::from(
                "For DHCP in AP mode, iwd should enable [General] EnableNetworkConfiguration=true.",
            )
            .style(Style::default().fg(Color::Gray)),
        ])
        .alignment(Alignment::Left)
        .wrap(ratatui::widgets::Wrap { trim: true }),
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(56),
                Constraint::Fill(1),
            ])
            .split(rows[5])[1],
    );
}
