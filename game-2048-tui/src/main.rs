use std::error::Error;
use std::io;

use crossterm::event::KeyCode;
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::Color;
use tui::widgets::canvas::{Canvas, Line, Points};
use tui::widgets::{Block, Borders};
use tui::Terminal;

use app::App;

use crate::event::{Events, GameEvent};
use crate::game::Command;

mod app;
mod event;
mod game;

/// thanks ['game2048']
///
/// ['game2048']:https://github.com/WanderHuang/game-2048-tui
fn main() -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::default();

    let events = Events::default();

    loop {
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
                    for row in 0..grid.len() {
                        for col in 0..grid.len() {
                            let score = grid[row][col];
                            let s = if score == 0 {
                                String::new().into_boxed_str()
                            } else {
                                score.to_string().into_boxed_str()
                            };

                            let x_box = (col as f64) * app.box_size;
                            let y_box = (row as f64) * app.box_size;
                            //数字从左上角开始
                            ctx.print(
                                ((col + 1) as f64) * app.box_size - half_box_size - font_width,
                                ((4 - row) as f64) * app.box_size
                                    - half_box_size
                                    - font_width * 2.0,
                                Box::leak(s),
                                score_to_color(score),
                            );

                            //框框从左下角开始
                            ctx.draw(&Line {
                                x1: x_box,
                                y1: y_box,
                                x2: x_box + app.box_size,
                                y2: y_box,
                                color: Color::Green,
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

                    if !app.is_alive() {
                        ctx.draw(&Points {
                            coords: &app.get_game_over_modal(),
                            color: Color::Green,
                        });

                        ctx.print(
                            app.box_size * 1.5,
                            app.box_size * 2.0,
                            " GAME OVER! ",
                            Color::Blue,
                        );

                        ctx.print(
                            app.box_size * 1.3,
                            app.box_size * 1.8,
                            " Restart[R] Quit[Q] ",
                            Color::Blue,
                        );
                    }
                })
                .x_bounds([0.0, board_size])
                .y_bounds([0.0, board_size]);
            f.render_widget(game_board, chunks[0]);

            let infos = Canvas::default()
                .block(Block::default().borders(Borders::ALL).title("Panel"))
                .paint(|ctx| {
                    ctx.print(board_size, board_size, "> Relax <", Color::Blue);

                    let score = app.get_score().to_string().into_boxed_str();
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

        match events.next()? {
            GameEvent::Input(input) => match input {
                KeyCode::Char('q') => {
                    break;
                }
                KeyCode::Char('r') => app.restart(),
                KeyCode::Down => {
                    app.add_command(Command::Down);
                }
                KeyCode::Up => {
                    app.add_command(Command::Up);
                }
                KeyCode::Right => {
                    app.add_command(Command::Right);
                }
                KeyCode::Left => {
                    app.add_command(Command::Left);
                }
                _ => {}
            },
            GameEvent::Tick => {
                app.next();
            }
        }
    }

    Ok(())
}

/// render different color for different score
fn score_to_color(score: u32) -> Color {
    match score {
        2 => Color::Green,
        4 => Color::Yellow,
        8 => Color::LightBlue,
        16 => Color::Blue,
        s if s < 64 => Color::White,
        s if s < 256 => Color::Magenta,
        s if s < 1024 => Color::Cyan,
        s if s < 4096 => Color::LightRed,
        _ => Color::Red,
    }
}
