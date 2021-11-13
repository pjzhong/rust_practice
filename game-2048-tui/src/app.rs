use crate::game::SQUARE;
use crate::game::{Game, Grid};

/// 2048 Application
///
/// Rules：
///
/// 1. make a `squares * squares` grid
/// 2. each square is same size
/// 3. board_size = box_size * X
///
/// :> TODO make each `config` as a input list so this game can be customized;
pub struct App {
    pub x: f64,
    pub y: f64,
    pub box_size: f64,
    score: u32,
    game: Game,
}

impl Default for App {
    fn default() -> Self {
        App::new()
    }
}

impl App {
    pub fn new() -> App {
        Self {
            x: 0.0,
            y: 0.0,
            box_size: 40.0,
            score: 0,
            game: Game::default(),
        }
    }

    pub fn get_size(&self) -> f64 {
        self.box_size * SQUARE as f64
    }

    pub fn get_grid(&self) -> Grid {
        self.game.get_grid()
    }
}
