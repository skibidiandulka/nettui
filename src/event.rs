use anyhow::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
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
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);

            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();

                tokio::select! {
                    () = sender_cloned.closed() => {
                        break;
                    }
                    _ = tick_delay => {
                        let _ = sender_cloned.send(Event::Tick);
                    }
                    Some(Ok(evt)) = crossterm_event => {
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
