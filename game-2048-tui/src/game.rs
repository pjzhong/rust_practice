use rand::Rng;

/// how many square
pub const SQUARE: usize = 4;

pub type Grid = [[u32; SQUARE]; SQUARE];

/// game command
pub enum Command {
    Left,
    Up,
    Right,
    Down,
}

pub struct Game {
    pub alive: bool,
    grid: [[u32; SQUARE]; SQUARE],
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
    pub fn get_score(&self) -> u32 {
        let mut sum = 0;
        for row in self.grid {
            for col in row {
                sum += col;
            }
        }
        sum
    }

    pub fn get_grid(&self) -> Grid {
        self.grid
    }

    pub fn start(&mut self) {
        self.random_insert();
        self.random_insert();
    }

    pub fn random_insert(&mut self) {
        let mut vec: Vec<(usize, usize)> = vec![];

        for i in 0..SQUARE {
            for j in 0..SQUARE {
                if self.grid[i][j] == 0 {
                    vec.push((i, j));
                }
            }
        }

        let len = vec.len();

        if len == 0 {
            return;
        }

        let rand_num: usize = rand::thread_rng().gen_range(0..len);
        let (i, j) = vec[rand_num];

        let rand_num = rand::thread_rng().gen_range(0..10);
        let val = if rand_num < 6 { 2 } else { 4 };

        self.grid[i][j] = val;
    }

    pub fn next_tick(&mut self, cmd: Command) {
        let grid_change = self.do_next_tick(cmd);
        self.alive = self.check_alive();
        if self.alive && grid_change {
            self.random_insert();
        }
    }

    fn do_next_tick(&mut self, cmd: Command) -> bool {
        let mut changed = false;
        match cmd {
            Command::Down => {
                for y in 0..self.grid.len() {
                    let (added, res) = sum(vec![
                        self.grid[0][y],
                        self.grid[1][y],
                        self.grid[2][y],
                        self.grid[3][y],
                    ]);

                    self.grid[0][y] = res[0];
                    self.grid[1][y] = res[1];
                    self.grid[2][y] = res[2];
                    self.grid[3][y] = res[3];

                    changed |= added;
                }
            }
            Command::Up => {
                for y in 0..self.grid.len() {
                    let (added, res) = sum(vec![
                        self.grid[3][y],
                        self.grid[2][y],
                        self.grid[1][y],
                        self.grid[0][y],
                    ]);

                    self.grid[0][y] = res[3];
                    self.grid[1][y] = res[2];
                    self.grid[2][y] = res[1];
                    self.grid[3][y] = res[0];

                    changed |= added;
                }
            }
            Command::Left => {
                for x in 0..self.grid.len() {
                    let (added, res) = sum(vec![
                        self.grid[x][3],
                        self.grid[x][2],
                        self.grid[x][1],
                        self.grid[x][0],
                    ]);

                    self.grid[x][0] = res[3];
                    self.grid[x][1] = res[2];
                    self.grid[x][2] = res[1];
                    self.grid[x][3] = res[0];

                    changed |= added;
                }
            }
            Command::Right => {
                for x in 0..self.grid.len() {
                    let (added, res) = sum(vec![
                        self.grid[x][0],
                        self.grid[x][1],
                        self.grid[x][2],
                        self.grid[x][3],
                    ]);

                    self.grid[x][0] = res[0];
                    self.grid[x][1] = res[1];
                    self.grid[x][2] = res[2];
                    self.grid[x][3] = res[3];

                    changed |= added;
                }
            }
        }

        changed
    }

    fn check_alive(&self) -> bool {
        let has_zero = self.grid.iter().any(|arr| arr.iter().any(|x| *x == 0));
        if has_zero {
            return has_zero;
        }

        for i in 0..SQUARE {
            for j in 0..SQUARE {
                let x = self.grid[i][j];

                let left = if j != 0 { self.grid[i][j - 1] } else { 0 };

                let up = if i != 0 { self.grid[i - 1][j] } else { 0 };

                let down = if i < SQUARE - 1 {
                    self.grid[i + 1][j]
                } else {
                    0
                };

                let right = if j < SQUARE - 1 {
                    self.grid[i][j + 1]
                } else {
                    0
                };

                if x == left || x == right || x == up || x == down {
                    return true;
                }
            }
        }

        false
    }
}

/// sum it
/// 1 1 2 2 -> 0 0 2 4
/// 2 2 2 2 -> 0 0 0 8
fn sum(mut arr: Vec<u32>) -> (bool, Vec<u32>) {
    let changed = do_sum(&mut arr);
    while do_sum(&mut arr) {}
    (changed, arr)
}

fn do_sum(arr: &mut Vec<u32>) -> bool {
    let mut changed = false;
    let mut idx = arr.len() - 1;
    while 0 < idx {
        let old = arr[idx];
        if arr[idx] == 0 && 0 < arr[idx - 1] {
            arr[idx] = arr[idx - 1];
            arr[idx - 1] = 0;
        } else if 0 < arr[idx] && arr[idx] == arr[idx - 1] {
            arr[idx] *= 2;
            arr[idx - 1] = 0;
        }

        changed |= arr[idx] != old;
        idx -= 1;
    }

    changed
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_sum() {
        use super::sum;

        assert_eq!((false, vec![0, 0, 0, 4]), sum(vec![0, 0, 0, 4]));
        assert_eq!((false, vec![1, 2, 3, 4]), sum(vec![1, 2, 3, 4]));
        assert_eq!((true, vec![0, 0, 2, 4]), sum(vec![1, 1, 2, 2]));
        assert_eq!((true, vec![0, 0, 0, 64]), sum(vec![0, 16, 16, 32]));
    }
}
