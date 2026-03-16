use super::popup_layout::centered_rect;
use crate::app::{App, WifiApPromptField, WifiAuthPromptField};
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
    if app.wifi_auth_prompt.is_some() {
        render_wifi_auth_popup(app, frame);
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

fn render_wifi_auth_popup(app: &App, frame: &mut Frame) {
    let Some(prompt) = app.wifi_auth_prompt.as_ref() else {
        return;
    };

    let has_secondary_field = prompt.has_secondary_field();
    let has_fixed_user_name = prompt.fixed_user_name().is_some();
    let area_height = if has_secondary_field || has_fixed_user_name {
        50
    } else {
        38
    };

    let area = centered_rect(62, area_height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(prompt.title())
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let primary_value = if prompt.primary_is_secret() && !prompt.primary_visible {
        "*".repeat(prompt.primary_input.chars().count())
    } else {
        prompt.primary_input.clone()
    };
    let secondary_value = if prompt.secondary_is_secret() && !prompt.secondary_visible {
        "*".repeat(prompt.secondary_input.chars().count())
    } else {
        prompt.secondary_input.clone()
    };
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(if has_secondary_field || has_fixed_user_name {
                12
            } else {
                8
            }),
            Constraint::Fill(1),
        ])
        .split(inner);
    let content_area = outer[1];
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(auth_popup_constraints(
            has_secondary_field,
            has_fixed_user_name,
        ))
        .split(content_area);
    let ssid_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(rows[0])[1];
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("SSID: ").bold(),
            Span::from(prompt.ssid.as_str()).fg(Color::Cyan),
        ])),
        ssid_area,
    );

    let mut row_index = 1;
    if let Some(user_name) = prompt.fixed_user_name() {
        let user_name_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(42),
                Constraint::Fill(1),
            ])
            .split(rows[row_index])[1];
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::from("Username: ").bold(),
                Span::from(user_name).fg(Color::Cyan),
            ])),
            user_name_area,
        );
        row_index += 1;
    }

    let primary_label_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(rows[row_index])[1];
    frame.render_widget(
        Paragraph::new(Line::from(prompt.primary_label()).style(Style::default().bold())),
        primary_label_area,
    );
    row_index += 1;

    let primary_field_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Length(42),
            Constraint::Fill(1),
        ])
        .split(rows[row_index])[1];
    let primary_block = Block::default()
        .borders(Borders::ALL)
        .border_style(
            Style::default().fg(if prompt.focus == WifiAuthPromptField::Primary {
                Color::Cyan
            } else {
                Color::Gray
            }),
        )
        .border_type(BorderType::Rounded);
    let primary_inner = primary_block.inner(primary_field_area);
    frame.render_widget(primary_block, primary_field_area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::from(primary_value))).alignment(Alignment::Center),
        primary_inner,
    );
    row_index += 1;

    if let Some(secondary_label) = prompt.secondary_label() {
        let secondary_label_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(42),
                Constraint::Fill(1),
            ])
            .split(rows[row_index])[1];
        frame.render_widget(
            Paragraph::new(Line::from(secondary_label).style(Style::default().bold())),
            secondary_label_area,
        );
        row_index += 1;

        let secondary_field_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Fill(1),
                Constraint::Length(42),
                Constraint::Fill(1),
            ])
            .split(rows[row_index])[1];
        let secondary_block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                Style::default().fg(if prompt.focus == WifiAuthPromptField::Secondary {
                    Color::Cyan
                } else {
                    Color::Gray
                }),
            )
            .border_type(BorderType::Rounded);
        let secondary_inner = secondary_block.inner(secondary_field_area);
        frame.render_widget(secondary_block, secondary_field_area);
        frame.render_widget(
            Paragraph::new(Line::from(Span::from(secondary_value))).alignment(Alignment::Center),
            secondary_inner,
        );
        row_index += 1;
    }

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from(if has_secondary_field {
                "↑/↓"
            } else {
                "⇥"
            })
            .bold(),
            Span::from(if has_secondary_field {
                " field"
            } else {
                " show/hide"
            }),
            Span::from(" | "),
            Span::from("⇥").bold(),
            Span::from(" show/hide"),
            Span::from(" | "),
            Span::from("↵").bold(),
            Span::from(" submit"),
            Span::from(" | "),
            Span::from("Esc").bold(),
            Span::from(" cancel"),
        ]))
        .alignment(Alignment::Center),
        rows[row_index],
    );
}

fn auth_popup_constraints(has_secondary_field: bool, has_fixed_user_name: bool) -> Vec<Constraint> {
    let mut constraints = vec![Constraint::Length(1)];
    if has_fixed_user_name {
        constraints.push(Constraint::Length(1));
    }
    constraints.push(Constraint::Length(1));
    constraints.push(Constraint::Length(3));
    if has_secondary_field {
        constraints.push(Constraint::Length(1));
        constraints.push(Constraint::Length(3));
    }
    constraints.push(Constraint::Length(1));
    constraints
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
