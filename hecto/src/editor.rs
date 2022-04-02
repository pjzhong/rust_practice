use crate::Document;
use crate::Row;
use crate::Terminal;
use std::env;
use std::io::Error;
use std::time::Duration;
use std::time::Instant;
use termion::color;
use termion::event::Key;

const STATUS_FG_COLOR: color::Rgb = color::Rgb(63, 63, 63);
const STATUS_BG_COLOR: color::Rgb = color::Rgb(239, 239, 239);
const VERSION: &str = env!("CARGO_PKG_VERSION");
const QUIT_TIMES: u8 = 3;

#[derive(PartialEq, Copy, Clone)]
pub enum SearchDirection {
    Forward,
    Backward,
}

#[derive(Default, Clone)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

struct StatusMessage {
    text: String,
    time: Instant,
}

impl StatusMessage {
    fn from(message: String) -> Self {
        Self {
            time: Instant::now(),
            text: message,
        }
    }
}

pub struct Editor {
    should_quit: bool,
    terminal: Terminal,
    document: Document,
    cursor_position: Position,
    offset: Position,
    status_message: StatusMessage,
    quit_times: u8,
}

impl Editor {
    pub fn run(&mut self) {
        loop {
            if let Err(error) = self.refresh_screen() {
                die(error);
            }

            if self.should_quit {
                break;
            }

            if let Err(error) = self.process_keypress() {
                die(error);
            }
        }
    }

    fn process_keypress(&mut self) -> Result<(), Error> {
        let pressed_key = Terminal::read_key()?;
        match pressed_key {
            Key::Ctrl('q') => {
                if self.quit_times > 0 && self.document.is_dirty() {
                    self.status_message = StatusMessage::from(format!(
                        "WARNING! File has unsaved changed. Press Ctrl-Q {} more times to quit.",
                        self.quit_times
                    ));
                    self.quit_times -= 1;
                    return Ok(());
                } else {
                    self.should_quit = true
                }
            }
            Key::Ctrl('s') => self.save(),
            Key::Ctrl('f') => self.search(),
            Key::Char(c) => {
                self.document.insert(&self.cursor_position, c);
                self.move_cursor(Key::Right);
            }
            Key::Delete => {
                self.document.delete(&self.cursor_position);
                self.move_cursor(Key::Left);
            }
            Key::Backspace => {
                if self.cursor_position.x > 0 || self.cursor_position.y > 0 {
                    self.move_cursor(Key::Left);
                    self.document.delete(&self.cursor_position);
                }
            }
            Key::Up
            | Key::Down
            | Key::Left
            | Key::Right
            | Key::PageUp
            | Key::PageDown
            | Key::End
            | Key::Home => self.move_cursor(pressed_key),
            _ => (),
        }
        if self.quit_times < QUIT_TIMES {
            self.quit_times = QUIT_TIMES;
            self.status_message = StatusMessage::from(String::new());
        }

        self.scroll();
        Ok(())
    }

    fn search(&mut self) {
        let old_position = self.cursor_position.clone();
        let query = self
            .prompt(
                "Search(ESC to cancel, Arrows to navigate):",
                |editor, key, query| {
                    let (moved, direction) = match key {
                        Key::Right | Key::Down => {
                            editor.move_cursor(Key::Right);
                            (true, SearchDirection::Forward)
                        }
                        Key::Left | Key::Up => (false, SearchDirection::Backward),
                        _ => (false, SearchDirection::Forward),
                    };
                    if let Some(position) =
                        editor
                            .document
                            .find(query, &editor.cursor_position, direction)
                    {
                        editor.cursor_position = position;
                        editor.scroll();
                    } else if moved {
                        editor.move_cursor(Key::Left);
                    }
                    editor.document.highlight(Some(query));
                },
            )
            .unwrap_or(None);

        if query.is_none() {
            self.cursor_position = old_position;
            self.scroll();
        }
        self.document.highlight(None);
    }

    fn move_cursor(&mut self, key: Key) {
        let Position { mut x, mut y } = self.cursor_position;
        let terminal_height = self.terminal.size().height as usize;
        let height = self.document.len();
        let mut width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };
        match key {
            Key::Up => y = y.saturating_sub(1),
            Key::Down => {
                if y < height {
                    y = y.saturating_add(1)
                }
            }
            Key::Left => {
                if x > 0 {
                    x -= 1;
                } else if y > 0 {
                    y -= 1;
                    x = if let Some(row) = self.document.row(y) {
                        row.len()
                    } else {
                        0
                    };
                }
            }
            Key::Right => {
                if x < width {
                    x += 1;
                } else if y < height {
                    y += 1;
                    x = 0;
                }
            }
            Key::PageUp => {
                y = if y > terminal_height {
                    y - terminal_height
                } else {
                    0
                }
            }
            Key::PageDown => {
                y = if y.saturating_add(terminal_height) < height {
                    y + terminal_height as usize
                } else {
                    height
                }
            }
            Key::Home => x = 0,
            Key::End => x = width,
            _ => (),
        };

        width = if let Some(row) = self.document.row(y) {
            row.len()
        } else {
            0
        };

        if x > width {
            x = width;
        }
        self.cursor_position.x = x;
        self.cursor_position.y = y;
    }

    fn scroll(&mut self) {
        let Position { x, y } = self.cursor_position;
        let width = self.terminal.size().width as usize;
        let height = self.terminal.size().height as usize;
        let mut offset = &mut self.offset;
        if y < offset.y {
            offset.y = y;
        } else if y >= offset.y.saturating_add(height) {
            offset.y = y.saturating_sub(height).saturating_add(1);
        }

        if x < offset.x {
            offset.x = x;
        } else if x >= offset.x.saturating_add(width) {
            offset.x = x.saturating_sub(width).saturating_add(1);
        }
    }

    fn refresh_screen(&self) -> Result<(), Error> {
        Terminal::cursor_hide();
        Terminal::cursor_position(&Position::default());
        if self.should_quit {
            Terminal::clear_screen();
            println!("Goodby.\r");
        } else {
            self.draw_rows();
            self.draw_status_bar();
            self.draw_message_bar();
            Terminal::cursor_position(&Position {
                x: self.cursor_position.x.saturating_sub(self.offset.x),
                y: self.cursor_position.y.saturating_sub(self.offset.y),
            });
        }
        Terminal::cursor_show();
        Terminal::flush()
    }

    fn draw_welcome_message(&self) {
        let mut welcome_message = format!("Hecto editor -- version {}", VERSION);
        let width = self.terminal.size().width as usize;
        let len = welcome_message.len();
        let padding = width.saturating_sub(len) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{}{}", spaces, welcome_message);
        welcome_message.truncate(width);
        println!("{}\r", welcome_message);
    }

    pub fn draw_row(&self, row: &Row) {
        let width = self.terminal.size().width as usize;
        let start = self.offset.x;
        let end = self.offset.x + width;
        let row = row.render(start, end);
        println!("{}\r", row);
    }

    fn draw_rows(&self) {
        let height = self.terminal.size().height;
        for row in 0..height {
            Terminal::clear_current_line();
            if let Some(r) = self.document.row(row as usize + self.offset.y) {
                self.draw_row(r);
            } else if self.document.is_empty() && row == height / 3 {
                self.draw_welcome_message();
            } else {
                println!("~\r");
            }
        }
    }

    fn draw_status_bar(&self) {
        let width = self.terminal.size().width as usize;
        let modified_indicator = if self.document.is_dirty() {
            " (modified)"
        } else {
            ""
        };

        let mut file_name = "[No Name]".to_string();
        if let Some(name) = &self.document.file_name {
            file_name = name.clone();
            file_name.truncate(20);
        }
        let mut status = format!(
            "{} - {} lines{}",
            file_name,
            self.document.len(),
            modified_indicator
        );

        let line_indicator = format!(
            "{}/{}",
            self.cursor_position.y.saturating_add(1),
            self.document.len()
        );

        let len = status.len() + line_indicator.len();
        if width > len {
            status.push_str(&" ".repeat(width - len));
        }

        status = format!("{}{}", status, line_indicator);
        status.truncate(width);
        Terminal::set_fg_color(STATUS_FG_COLOR);
        Terminal::set_bg_color(STATUS_BG_COLOR);
        println!("{}\r", status);
        Terminal::reset_fg_color();
        Terminal::reset_bg_color();
    }

    fn draw_message_bar(&self) {
        Terminal::clear_current_line();
        let message = &self.status_message;
        if Instant::now() - message.time < Duration::new(5, 0) {
            let mut text = message.text.clone();
            text.truncate(self.terminal.size().width as usize);
            print!("{}", text);
        }
    }

    fn save(&mut self) {
        if self.document.file_name.is_none() {
            let new_name = self.prompt("Save as: ", |_, _, _| {}).unwrap_or(None);
            if new_name.is_none() {
                self.status_message = StatusMessage::from("Save aborted.".to_string());
                return;
            }
            self.document.file_name = new_name;
        }

        if self.document.save().is_ok() {
            self.status_message = StatusMessage::from("File saved successfully.".to_string());
        } else {
            self.status_message = StatusMessage::from("Error writing file!".to_string());
        }
    }

    fn prompt<C>(&mut self, prompt: &str, callback: C) -> Result<Option<String>, Error>
    where
        C: Fn(&mut Self, Key, &String),
    {
        let mut result = String::new();
        loop {
            self.status_message = StatusMessage::from(format!("{}{}", prompt, result));
            self.refresh_screen()?;

            let key = Terminal::read_key()?;
            match key {
                Key::Backspace => {
                    if !result.is_empty() {
                        result.truncate(result.len() - 1);
                    }
                }
                Key::Char('\n') => break,
                Key::Char(c) => {
                    if !c.is_control() {
                        result.push(c);
                    }
                }
                Key::Esc => {
                    result.truncate(0);
                    break;
                }
                _ => (),
            }
            callback(self, key, &result);
        }
        self.status_message = StatusMessage::from(String::new());

        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

impl Default for Editor {
    fn default() -> Self {
        let mut args = env::args();
        let mut initial_status =
            String::from("HELP: Ctrl-F = find | Ctrl-S = save | Ctrl-Q = quit");
        let _ = args.next();
        let document = if let Some(file_name) = args.next() {
            let doc = if let Ok(doc) = Document::open(&file_name) {
                doc
            } else {
                initial_status = format!("ERR: Could not open file: {}", file_name);
                Document::default()
            };
            doc
        } else {
            Document::default()
        };

        Self {
            should_quit: false,
            terminal: Terminal::default().expect("Failed to initialize terminal"),
            document,
            cursor_position: Position::default(),
            offset: Position::default(),
            status_message: StatusMessage::from(initial_status),
            quit_times: QUIT_TIMES,
        }
    }
}

fn die(e: std::io::Error) {
    Terminal::clear_screen();
    panic!("{}", e);
}
