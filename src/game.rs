use crate::render::{self};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum BoardError {
    #[error("invalid seed separator: {0}")]
    InvalidSeparator(char),
    #[error("invalid seed character: '{0}', expected '{1}' or '{2}'")]
    InvalidSeedCharacter(char, char, char),
}

const NEIGHBORS: [(isize, isize); 8] = [
    (-1, -1), // NW
    (-1, 0),  // N
    (-1, 1),  // NE
    (0, 1),   // E
    (1, 1),   // SE
    (1, 0),   // S
    (1, -1),  // SW
    (0, -1),  // W
];

pub struct Game {
    pub board: Board,
    pub generation: usize,
    pub delta: usize,
}

impl From<Board> for Game {
    fn from(board: Board) -> Self {
        Game {
            board,
            generation: 0,
            delta: 0,
        }
    }
}

impl Game {
    pub fn next(&mut self) {
        self.delta = self.board.next() as usize;
        self.generation += 1;
    }

    pub fn is_terminal(&self) -> bool {
        self.generation != 0 && self.delta == 0
    }
}

impl std::fmt::Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[n: {}, Δ: {}] \n", self.generation, self.delta,)?;
        write!(f, "{}", render::text(&self, render::TextOptions::default()))
    }
}

pub struct Board {
    pub grid: Vec<Vec<bool>>,
}

impl TryFrom<String> for Board {
    type Error = BoardError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let opts = render::TextOptions::default();
        Board::from_seed(value, opts.alive, opts.dead, opts.separator)
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let o = render::TextOptions::default();
        write!(f, "{}", self.stringify(o.alive, o.dead, o.separator))
    }
}

impl Board {
    pub fn from_seed(
        seed: String,
        alive: char,
        dead: char,
        separator: char,
    ) -> Result<Self, BoardError> {
        if separator == alive || separator == dead {
            return Err(BoardError::InvalidSeparator(separator));
        }

        let seeds = seed.trim().split(separator).collect::<Vec<&str>>();
        let cols = seeds.iter().map(|s| s.len()).max().unwrap_or(0);

        let mut grid = vec![vec![false; cols]; seeds.len()];
        for (row_idx, row_seed) in seeds.into_iter().enumerate() {
            for (col_idx, cell) in row_seed.char_indices() {
                if cell == alive {
                    grid[row_idx][col_idx] = true;
                } else if cell != dead {
                    return Err(BoardError::InvalidSeedCharacter(cell, alive, dead));
                }
            }
        }

        Ok(Board { grid })
    }

    pub fn stringify(&self, alive: char, dead: char, separator: char) -> String {
        let mut result = String::with_capacity(self.rows() * self.cols() + self.rows());

        for (i, row) in self.grid.iter().enumerate() {
            for cell in row {
                result.push(if *cell { alive } else { dead });
            }
            if i < self.rows() - 1 {
                result.push(separator);
            }
        }

        result
    }

    pub fn next(&mut self) -> i32 {
        let mut next = self.grid.clone();
        let mut delta = 0;

        for row in 0..self.grid.len() {
            for col in 0..self.grid[row].len() {
                let (next_state, has_changed) = self.interact(row, col);
                if has_changed {
                    delta += 1;
                }
                next[row][col] = next_state
            }
        }

        self.grid = next;
        delta
    }

    pub fn rows(&self) -> usize {
        self.grid.len()
    }

    pub fn cols(&self) -> usize {
        self.grid[0].len()
    }

    fn safe_get(&self, row: isize, col: isize) -> bool {
        if row < 0 || col < 0 {
            return false;
        }

        if let Some(r) = self.grid.get(row as usize) {
            if let Some(cell) = r.get(col as usize) {
                return *cell;
            }
        }

        false
    }

    fn interact(&self, row: usize, col: usize) -> (bool, bool) {
        let neighbors = self.neighbors(row, col);
        let alive = self.safe_get(row as isize, col as isize);

        let next = match (neighbors, alive) {
            // Any dead cell with exactly three live neighbors becomes a live cell, as if by reproduction.
            (3, false) => true,
            // Any live cell with fewer than two live neighbors dies.
            (0..=1, true) => false,
            // Any live cell with two or three live neighbors lives on to the next generation.
            (2..=3, true) => true,
            // Any live cell with more than three live neighbors dies. Or, a dead cell stays dead.
            (_, _) => false,
        };

        (next, next != alive)
    }

    fn neighbors(&self, row: usize, col: usize) -> usize {
        NEIGHBORS
            .iter()
            .filter(|(r, c)| self.safe_get(row as isize + r, col as isize + c))
            .count()
    }
}
