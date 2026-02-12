use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone, Debug)]
pub enum Event {
    Tick,
    Key(KeyEvent),
    Resize(u16, u16),
}

#[derive(Debug)]
pub struct EventHandler {
    pub sender: mpsc::UnboundedSender<Event>,
    pub receiver: mpsc::UnboundedReceiver<Event>,
    _handler: tokio::task::JoinHandle<()>,
}

impl EventHandler {
    pub fn new(tick_rate_ms: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate_ms);
        let (sender, receiver) = mpsc::unbounded_channel();
        let sender_cloned = sender.clone();

        let handler = tokio::spawn(async move {
            let mut next_tick = std::time::Instant::now() + tick_rate;

            loop {
                if sender_cloned.is_closed() {
                    break;
                }

                // Poll terminal events in short slices to keep UI responsive and avoid
                // EventStream backend panics on some environments.
                match crossterm::event::poll(Duration::from_millis(25)) {
                    Ok(true) => {
                        if let Ok(evt) = crossterm::event::read() {
                            match evt {
                                CrosstermEvent::Key(key) => {
                                    if key.kind == crossterm::event::KeyEventKind::Press {
                                        let _ = sender_cloned.send(Event::Key(key));
                                    }
                                }
                                CrosstermEvent::Resize(x, y) => {
                                    let _ = sender_cloned.send(Event::Resize(x, y));
                                }
                                _ => {}
                            }
                        }
                    }
                    Ok(false) => {}
                    Err(_) => {}
                }

                let now = std::time::Instant::now();
                if now >= next_tick {
                    let _ = sender_cloned.send(Event::Tick);
                    next_tick = now + tick_rate;
                }
            }
        });

        Self {
            sender,
            receiver,
            _handler: handler,
        }
    }

    pub async fn next(&mut self) -> Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_else(|| std::io::Error::other("event stream closed").into())
    }
}
