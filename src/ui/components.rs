use crate::{
    app::App,
    domain::common::{ActiveTab, ToastKind},
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
    let mut spans = vec![
        Span::from("Tab").bold(),
        Span::from(" switch"),
        Span::from(" | "),
        Span::from("k,↑").bold(),
        Span::from(" Up"),
        Span::from(" | "),
        Span::from("j,↓").bold(),
        Span::from(" Down"),
        Span::from(" | "),
        Span::from("r").bold(),
        Span::from(" refresh"),
        Span::from(" | "),
    ];

    match app.active_tab {
        ActiveTab::Wifi => {
            spans.extend([
                Span::from("s").bold(),
                Span::from(" scan"),
                Span::from(" | "),
                Span::from("Enter").bold(),
                Span::from(" connect/disconnect"),
            ]);
        }
        ActiveTab::Ethernet => {
            spans.extend([Span::from("n").bold(), Span::from(" renew DHCP")]);
        }
    }

    spans.extend([
        Span::from(" | "),
        Span::from("q").bold(),
        Span::from(" quit"),
    ]);

    let p = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Cyan));
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

    let area = centered_rect(80, 28, frame.area());
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
