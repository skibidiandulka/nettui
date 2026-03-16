// Copyright (C) 2026 skibidiandulka
// Clean-room implementation inspired by Impala UX by pythops.

mod auth_popups;
mod info_popups;
mod panels;
mod popup_layout;
mod styles;

use crate::app::App;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

pub fn render(app: &mut App, frame: &mut Frame, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(7),
            Constraint::Min(6),
            Constraint::Length(5),
        ])
        .split(area);

    panels::render_known_networks(app, frame, chunks[0]);
    panels::render_new_networks(app, frame, chunks[1]);
    panels::render_device(app, frame, chunks[2]);

    info_popups::render_info_popups(app, frame);
    auth_popups::render_auth_popups(app, frame);
}
