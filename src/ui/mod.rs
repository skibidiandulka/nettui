mod components;
mod ethernet;
mod layout;
mod wifi;

use crate::app::App;
use ratatui::Frame;

pub fn render(app: &mut App, frame: &mut Frame) {
    layout::render(app, frame);
}
