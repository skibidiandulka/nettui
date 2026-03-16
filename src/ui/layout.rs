use crate::{
    app::App,
    domain::common::ActiveTab,
    ui::{components, ethernet, wifi},
};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    const MIN_W: u16 = 119;
    const MIN_H: u16 = 35;

    let area = frame.area();
    if area.width < MIN_W || area.height < MIN_H {
        components::render_too_small(frame, area, MIN_W, MIN_H);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // tabs
            Constraint::Min(10),   // content
            Constraint::Length(3), // footer
        ])
        .margin(1)
        .split(area);

    components::render_tabs(app, frame, chunks[0]);

    match app.active_tab {
        ActiveTab::Wifi => wifi::render(app, frame, chunks[1]),
        ActiveTab::Ethernet => ethernet::render(app, frame, chunks[1]),
    }

    if app.wifi_auth_prompt.is_none() && !app.wifi_ap_prompt_open && app.wifi_share_popup.is_none()
    {
        components::render_footer(app, frame, chunks[2]);
    }

    for (index, toast) in app.visible_toasts().iter().enumerate() {
        components::render_toast_popup(frame, toast.kind, &toast.msg, index as u16);
    }
}
