use crate::{app::App, event::EventHandler, ui};
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::Backend};
use std::io;

pub struct Tui<B: Backend> {
    pub terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> Result<()> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<()> {
        self.terminal.draw(|frame| ui::render(app, frame))?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<()> {
        disable_raw_mode()?;
        execute!(io::stdout(), LeaveAlternateScreen)?;
        Ok(())
    }
}
