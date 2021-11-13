/// how many square
pub const SQUARE: usize = 4;

pub type Grid = [[i32; SQUARE]; SQUARE];

pub struct Game {
    alive: bool,
    grid: [[i32; SQUARE]; SQUARE],
}

impl Default for Game {
    fn default() -> Self {
        Self {
            alive: true,
            grid: [[0; SQUARE]; SQUARE],
        }
    }
}

impl Game {
    pub fn get_grid(&self) -> Grid {
        self.grid
    }
}
