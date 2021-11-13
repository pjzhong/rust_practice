use std::error::Error;
use std::io;
use std::thread::sleep;
use std::time::Duration;

use rand::thread_rng;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Line};
use tui::widgets::{Block, Borders};
use tui::Terminal;

use app::App;

use crate::game::SQUARE;

mod app;
mod game;

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    terminal.draw(|f| {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
            .split(f.size());

        let board_size = app.get_size();
        let panel_size = board_size + (board_size / 3.0);
        let half_box_size = app.box_size / 2.0;
        let font_width = 2.0;
        let game_board = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("2048-@wander"))
            .paint(|ctx| {
                let grid = app.get_grid();
                let mut i = 0;
                for row in 0..SQUARE {
                    for col in 0..SQUARE {
                        i += 1;
                        let score = grid[row][col];
                        let s = if score == 0 {
                            i.to_string().into_boxed_str()
                        } else {
                            score.to_string().into_boxed_str()
                        };

                        let x_box = (col as f64) * app.box_size;
                        let y_box = (row as f64) * app.box_size;
                        //数字从左上角开始
                        ctx.print(
                            ((col + 1) as f64) * app.box_size - half_box_size - font_width,
                            ((4 - row) as f64) * app.box_size - half_box_size - font_width * 2.0,
                            Box::leak(s),
                            score_to_color(score),
                        );

                        //框框从左下角开始
                        ctx.draw(&Line {
                            x1: x_box,
                            y1: y_box,
                            x2: x_box + app.box_size,
                            y2: y_box,
                            color: Color::Red,
                        });

                        ctx.draw(&Line {
                            x1: x_box,
                            y1: y_box,
                            x2: x_box,
                            y2: y_box + app.box_size,
                            color: Color::Green,
                        });
                        ctx.draw(&Line {
                            x1: x_box + app.box_size,
                            y1: y_box,
                            x2: x_box + app.box_size,
                            y2: y_box + app.box_size,
                            color: Color::Green,
                        });
                        ctx.draw(&Line {
                            x1: x_box,
                            y1: y_box + app.box_size,
                            x2: x_box + app.box_size,
                            y2: y_box + app.box_size,
                            color: Color::Green,
                        });
                    }
                }
            })
            .x_bounds([0.0, board_size])
            .y_bounds([0.0, board_size]);
        f.render_widget(game_board, chunks[0]);

        let infos = Canvas::default()
            .block(Block::default().borders(Borders::ALL).title("Panel"))
            .paint(|ctx| {
                ctx.print(board_size, board_size, "> Relax <", Color::Blue);

                let score = String::from("30").into_boxed_str();
                ctx.print(board_size, board_size - 30.0, "score:", Color::Green);
                ctx.print(
                    board_size,
                    board_size - 50.0,
                    Box::leak(score),
                    Color::Green,
                );

                ctx.print(board_size, 0.0, "Quit[Q]", Color::Blue);
            })
            .x_bounds([board_size, panel_size])
            .y_bounds([0.0, board_size]);
        f.render_widget(infos, chunks[1]);
    })?;

    sleep(Duration::from_secs(10));
    Ok(())
}

/// render different color for different score
fn score_to_color(score: i32) -> Color {
    if score < 64 {
        Color::Green
    } else if score < 256 {
        Color::Magenta
    } else if score < 1024 {
        Color::Cyan
    } else if score < 4096 {
        Color::LightRed
    } else {
        Color::Red
    }
}
