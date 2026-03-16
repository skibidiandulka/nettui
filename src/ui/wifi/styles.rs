use ratatui::{
    style::{Color, Style, Stylize},
    widgets::{Block, BorderType, Borders},
};

pub(super) fn section_block(title: &str, focused: bool) -> Block<'_> {
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

pub(super) fn secondary_row_style() -> Style {
    Style::default().fg(Color::Gray)
}

pub(super) fn secondary_text_style() -> Style {
    Style::default().fg(Color::Gray).bold()
}
