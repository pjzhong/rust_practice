use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{Event, KeyCode};

pub enum GameEvent<I> {
    Input(I),
    Tick,
}

pub struct Events {
    rx: mpsc::Receiver<GameEvent<KeyCode>>,
    pub tick_rate: Duration,
}

impl Default for Events {
    fn default() -> Self {
        Events::new(Duration::from_millis(250))
    }
}

impl Events {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = mpsc::channel();

        let _ = {
            let tx = tx.clone();
            thread::spawn(move || {
                use crossterm::event::{poll, read};
                loop {
                    if poll(tick_rate).is_ok() {
                        if let Ok(Event::Key(key)) = read() {
                            if tx.send(GameEvent::Input(key.code)).is_err() {
                                return;
                            }
                        } else {
                        }
                    }
                }
            })
        };

        let _ = {
            thread::spawn(move || loop {
                if let Err(err) = tx.send(GameEvent::Tick) {
                    eprintln!("{}", err);
                    break;
                }
                thread::sleep(tick_rate);
            })
        };

        Events { rx, tick_rate }
    }

    pub fn next(&self) -> Result<GameEvent<KeyCode>, mpsc::RecvError> {
        self.rx.recv()
    }
}
