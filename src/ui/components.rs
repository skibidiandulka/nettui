use crate::{
    app::App,
    domain::common::{ActiveTab, ToastKind, WifiFocus},
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Tabs},
};

pub fn render_tabs(app: &App, frame: &mut Frame, area: Rect) {
    let titles = [" Wi-Fi ", " Ethernet "];
    let selected = match app.active_tab {
        ActiveTab::Wifi => 0,
        ActiveTab::Ethernet => 1,
    };

    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .title(" nettui ")
                .borders(Borders::ALL)
                .border_type(BorderType::Thick)
                .border_style(Style::default().fg(Color::Green)),
        )
        .select(selected)
        .highlight_style(Style::default().fg(Color::Cyan).bold())
        .divider("|");

    frame.render_widget(tabs, area);
}

pub fn render_footer(app: &App, frame: &mut Frame, area: Rect) {
    let prev_tab = app.keybinds.prev_tab.to_string();
    let next_tab = app.keybinds.next_tab.to_string();
    let up = app.keybinds.up.to_string();
    let down = app.keybinds.down.to_string();
    let refresh = app.keybinds.refresh.to_string();
    let quit = app.keybinds.quit.to_string();
    let wifi_scan = app.keybinds.wifi_scan.to_string();
    let wifi_show_all = app.keybinds.wifi_show_all.to_string();
    let wifi_forget = app.keybinds.wifi_forget.to_string();
    let wifi_autoconnect = app.keybinds.wifi_autoconnect.to_string();
    let wifi_hidden = app.keybinds.wifi_hidden.to_string();
    let wifi_details = app.keybinds.wifi_details.to_string();
    let ethernet_renew = app.keybinds.ethernet_renew.to_string();

    let mut line1 = vec![
        Span::from(format!("{prev_tab},←")).bold(),
        Span::from(" Prev tab"),
        Span::from(" | "),
        Span::from(format!("{next_tab},→")).bold(),
        Span::from(" Next tab"),
        Span::from(" | "),
        Span::from(format!("{up},↑")).bold(),
        Span::from(" Up"),
        Span::from(" | "),
        Span::from(format!("{down},↓")).bold(),
        Span::from(" Down"),
        Span::from(" | "),
        Span::from(refresh).bold(),
        Span::from(" refresh"),
    ];
    if app.active_tab == ActiveTab::Wifi {
        line1.extend([
            Span::from(" | "),
            Span::from("⇥").bold(),
            Span::from(" section"),
        ]);
    }

    let mut line2: Vec<Span> = Vec::new();
    match app.active_tab {
        ActiveTab::Wifi => match app.wifi_focus {
            WifiFocus::KnownNetworks => line2.extend([
                Span::from("↵").bold(),
                Span::from(" dis/connect"),
                Span::from(" | "),
                Span::from(wifi_show_all.clone()).bold(),
                Span::from(" show all"),
                Span::from(" | "),
                Span::from(wifi_forget).bold(),
                Span::from(" forget"),
                Span::from(" | "),
                Span::from(wifi_autoconnect).bold(),
                Span::from(" autoconnect"),
                Span::from(" | "),
                Span::from(wifi_scan.clone()).bold(),
                Span::from(" scan"),
            ]),
            WifiFocus::NewNetworks => line2.extend([
                Span::from("↵").bold(),
                Span::from(" connect"),
                Span::from(" | "),
                Span::from(wifi_show_all).bold(),
                Span::from(" show all"),
                Span::from(" | "),
                Span::from(wifi_hidden).bold(),
                Span::from(" hidden"),
                Span::from(" | "),
                Span::from(wifi_scan).bold(),
                Span::from(" scan"),
            ]),
            WifiFocus::Adapter => line2.extend([
                Span::from(wifi_scan).bold(),
                Span::from(" scan"),
                Span::from(" | "),
                Span::from(wifi_details).bold(),
                Span::from(" details"),
            ]),
        },
        ActiveTab::Ethernet => {
            line2.extend([
                Span::from("↵").bold(),
                Span::from(" link up/down"),
                Span::from(" | "),
                Span::from(ethernet_renew).bold(),
                Span::from(" renew DHCP"),
            ]);
        }
    }

    line2.extend([
        Span::from(" | "),
        Span::from(quit).bold(),
        Span::from(" quit"),
    ]);

    let p = Paragraph::new(vec![Line::from(line1), Line::from(line2)])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Blue));
    frame.render_widget(p, area);
}

pub fn render_error_popup(frame: &mut Frame, msg: &str) {
    let area = centered_rect(80, 40, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(" Error ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Red));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let p = Paragraph::new(msg)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    frame.render_widget(p, inner);
}

pub fn render_toast_popup(frame: &mut Frame, kind: ToastKind, msg: &str) {
    let (title, color) = match kind {
        ToastKind::Success => (" Success ", Color::Green),
        ToastKind::Error => (" Error ", Color::Red),
        ToastKind::Info => (" Info ", Color::Cyan),
    };

    let lines = msg.lines().count().max(1) as u16;
    let width = frame.area().width.saturating_sub(2).clamp(24, 58);
    let height = (lines + 2).clamp(4, 8);
    let area = top_right_rect(width, height, frame.area());
    frame.render_widget(Clear, area);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let p = Paragraph::new(msg)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: false });

    frame.render_widget(p, inner);
}

pub fn render_too_small(frame: &mut Frame, area: Rect, min_w: u16, min_h: u16) {
    let block = Block::default()
        .title(" nettui ")
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(Color::Yellow));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let msg = format!(
        "Terminal is too small.\\n\\nMinimum size: {}x{}\\nCurrent size:  {}x{}",
        min_w, min_h, area.width, area.height
    );

    let p = Paragraph::new(msg)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });
    frame.render_widget(p, inner);
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

fn top_right_rect(width: u16, height: u16, area: Rect) -> Rect {
    let margin_x: u16 = 1;
    let margin_y: u16 = 1;
    let width = width.min(area.width.saturating_sub(margin_x.saturating_mul(2)));
    let height = height.min(area.height.saturating_sub(margin_y.saturating_mul(2)));
    let x = area
        .x
        .saturating_add(area.width.saturating_sub(width + margin_x));
    let y = area.y.saturating_add(margin_y);
    Rect::new(x, y, width, height)
}
