use super::popup_layout::centered_rect;
use crate::{
    app::{App, WifiEnterprisePromptField},
    domain::wifi_enterprise::EnterpriseMethod,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

pub(super) fn render_enterprise_popup(app: &App, frame: &mut Frame) {
    let Some(prompt) = app.wifi_enterprise_prompt.as_ref() else {
        return;
    };

    let area = centered_rect(78, 74, frame.area());
    frame.render_widget(Clear, area);

    let title = if prompt.editing_existing {
        " Enterprise Wi-Fi Edit "
    } else {
        " Enterprise Wi-Fi Setup "
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Blue));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(2),
            Constraint::Length(3),
        ])
        .split(inner);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("SSID: ").bold(),
            Span::from(prompt.ssid.as_str()).fg(Color::Cyan),
        ])),
        chunks[0],
    );

    let visible_fields = prompt.visible_fields();
    let items = visible_fields
        .iter()
        .map(|field| ListItem::new(render_field_line(prompt, *field)))
        .collect::<Vec<_>>();
    let selected = visible_fields
        .iter()
        .position(|field| *field == prompt.focus)
        .unwrap_or(0);
    let mut state = ListState::default().with_selected(Some(selected));

    let list = List::new(items)
        .highlight_style(Style::default().bg(Color::DarkGray).fg(Color::White))
        .highlight_symbol(">")
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray)),
        );
    frame.render_stateful_widget(list, chunks[1], &mut state);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::from("↑/↓").bold(),
            Span::from(" field"),
            Span::from(" | "),
            Span::from("←/→").bold(),
            Span::from(" option"),
            Span::from(" | "),
            Span::from("⇥").bold(),
            Span::from(" show/hide"),
            Span::from(" | "),
            Span::from("↵").bold(),
            Span::from(prompt.submit_action_label()),
            Span::from(" | "),
            Span::from("Esc").bold(),
            Span::from(" cancel"),
        ]))
        .alignment(Alignment::Center),
        chunks[2],
    );

    frame.render_widget(
        Paragraph::new(vec![
            Line::from("Root access may be required to save the .8021x profile.")
                .style(Style::default().fg(Color::Yellow)),
            Line::from(match prompt.profile.method {
                EnterpriseMethod::Eduroam => "Eduroam uses a PEAP profile with MSCHAPV2 phase 2.",
                EnterpriseMethod::Peap
                | EnterpriseMethod::Ttls
                | EnterpriseMethod::Tls
                | EnterpriseMethod::Pwd => {
                    "Choose the EAP method and fill only the fields your network requires."
                }
            })
            .style(Style::default().fg(Color::Gray)),
        ])
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true }),
        chunks[3],
    );
}

fn render_field_line(
    prompt: &crate::app::WifiEnterprisePrompt,
    field: WifiEnterprisePromptField,
) -> Line<'static> {
    let (label, value) = match field {
        WifiEnterprisePromptField::Method => ("Method", prompt.profile.method.label().to_string()),
        WifiEnterprisePromptField::Identity => ("Identity", prompt.profile.identity.clone()),
        WifiEnterprisePromptField::ServerDomainMask => {
            ("Server domain", prompt.profile.server_domain_mask.clone())
        }
        WifiEnterprisePromptField::CaCert => ("CA cert", prompt.profile.ca_cert.clone()),
        WifiEnterprisePromptField::ClientCert => {
            ("Client cert", prompt.profile.client_cert.clone())
        }
        WifiEnterprisePromptField::ClientKey => ("Client key", prompt.profile.client_key.clone()),
        WifiEnterprisePromptField::KeyPassphrase => (
            "Key passphrase",
            mask_if_hidden(
                &prompt.profile.key_passphrase,
                prompt.key_passphrase_visible,
            ),
        ),
        WifiEnterprisePromptField::Phase2Method => (
            "Phase 2 method",
            prompt.profile.phase2_method.label().to_string(),
        ),
        WifiEnterprisePromptField::Phase2Identity => {
            ("Phase 2 identity", prompt.profile.phase2_identity.clone())
        }
        WifiEnterprisePromptField::Phase2Password => (
            "Phase 2 password",
            mask_if_hidden(
                &prompt.profile.phase2_password,
                prompt.phase2_password_visible,
            ),
        ),
        WifiEnterprisePromptField::Password => (
            "Password",
            mask_if_hidden(&prompt.profile.password, prompt.password_visible),
        ),
    };

    let display = if value.is_empty() {
        Span::from("-").style(Style::default().fg(Color::DarkGray))
    } else {
        Span::from(value)
    };

    Line::from(vec![
        Span::from(format!("{label:<16}")).bold(),
        Span::from(" "),
        display,
    ])
}

fn mask_if_hidden(value: &str, visible: bool) -> String {
    if visible {
        value.to_string()
    } else {
        "*".repeat(value.chars().count())
    }
}
