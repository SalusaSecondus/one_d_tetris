use core::time;
use std::{
    error::Error,
    io::{Write, stdout},
};

use crossterm::{
    cursor::MoveTo,
    queue,
    style::{
        Color::{self},
        Print, SetForegroundColor,
    },
};
use num_format::{Locale, ToFormattedString as _};
use rand::{RngExt, rngs::SmallRng};

use crate::display::{
    BLACK, BLUE, BLUE_GREEN, GRAY, GREEN, RED, RED_BLUE, RED_GREEN, RenderCell, WHITE, blend,
    collides,
};

#[derive(Debug, Clone)]
pub struct Piece {
    pub cells: Vec<Color>,
    pub pos: usize,
}

impl Piece {
    pub fn rotate(&mut self) {
        self.cells.rotate_left(1);
    }

    fn in_piece(&self, row: usize) -> bool {
        row >= self.pos && row < self.pos + self.cells.len()
    }
}

pub struct Board {
    pub curr_piece: Option<Piece>,
    pub cells: Vec<Color>,
    pub color_blind: bool,
}

impl Board {
    pub fn display(&self, min_row: u16, min_col: u16) -> Result<(), Box<dyn Error>> {
        // display::pos(min_row, min_col);
        // queue!(
        //     stdout(),
        //     BeginSynchronizedUpdate,
        //     MoveTo(min_col, min_row),
        //     SetForegroundColor(GRAY),
        //     Print("╓ ╖")
        // )?;
        for (offset, color) in self.cells.iter().enumerate() {
            // sleep(Duration::from_secs(1));
            // display::pos(min_row + offset + 1, min_col + 5);
            let center = if let Some(piece) = &self.curr_piece {
                if piece.in_piece(offset) {
                    blend(piece.cells[offset - piece.pos], *color)
                } else {
                    *color
                }
            } else {
                *color
            };

            let walls = if offset == 0 {
                ("╓", "╖")
            } else {
                ("║", "║")
            };
            queue!(
                stdout(),
                MoveTo(min_col, min_row + (offset as u16)),
                SetForegroundColor(GRAY),
                Print(walls.0),
                // SetForegroundColor(center),
                RenderCell(center, self.color_blind),
                // Print("▓"),
                SetForegroundColor(GRAY),
                Print(walls.1),
            )?;
            // display::pos(min_row + offset, min_col);
            // print!("{}#{}▓{}#", GRAY, center, GRAY);
        }

        queue!(
            stdout(),
            MoveTo(min_col, min_row + (self.cells.len() as u16)),
            SetForegroundColor(GRAY),
            Print("╚═╝")
        )?;
        stdout().flush()?;
        Ok(())
    }

    pub fn drop(&mut self) -> bool {
        if self.curr_piece.is_none() {
            return false;
        }
        let curr_piece = self.curr_piece.as_mut().unwrap();
        let piece_len = curr_piece.cells.len();

        // First check if we're at the bottom
        if curr_piece.pos + piece_len >= self.cells.len() {
            return false;
        }

        // Check each item of the piece to see if it can move
        for (offset, cell) in curr_piece.cells.iter().enumerate() {
            if collides(self.cells[curr_piece.pos + offset + 1], *cell) {
                return false;
            }
        }
        curr_piece.pos += 1;
        true
    }

    pub fn merge_piece(&mut self) {
        if self.curr_piece.is_none() {
            return;
        }
        let curr_piece = self.curr_piece.take().unwrap();
        for (offset, color) in curr_piece.cells.iter().enumerate() {
            self.cells[curr_piece.pos + offset] =
                blend(self.cells[curr_piece.pos + offset], *color);
        }
    }
    pub fn clear_lines(&mut self) -> u32 {
        let mut cnt = 0;
        for pos in 0..self.cells.len() {
            // write!(stderr(), "{:?}\n", self.cells[pos]);
            if self.cells[pos] == WHITE {
                cnt += 1;
                self.cells[pos] = BLACK;
                // println!("Position = {}", pos);
                for dst in (1..=pos).rev() {
                    // println!("Copying from {} to {}", dst - 1, dst);
                    self.cells[dst] = self.cells[dst - 1];
                    self.cells[dst - 1] = BLACK;
                }
            }
        }
        cnt
    }
}

pub struct Game {
    pub lines: u32,
    pub score: u32,
    pub board: Board,
    next_piece: Option<Piece>,
    rng: SmallRng,
}

const PIECE_COLORS: [crossterm::style::Color; 3] = [RED, GREEN, BLUE];
// TODO: Figure out proper sizes
const MAX_PIECE_SIZE: usize = 3;
const START_ROW: u16 = 0;
const START_COL: u16 = 0;
const BOARD_HEIGHT: u16 = 32;
pub const GAME_HEIGHT: u16 = BOARD_HEIGHT + 1;
pub const GAME_WIDTH: u16 = 35; 

impl Game {
    pub fn new() -> Self {
        let mut rng = rand::make_rng();
        let board = Board {
            curr_piece: None,
            cells: vec![BLACK; BOARD_HEIGHT as usize],
            color_blind: false,
        };
        let next_piece = rand_piece(&mut rng);
        Self {
            lines: 0,
            score: 0,
            board,
            next_piece: Some(next_piece),
            rng,
        }
    }
    pub fn toggle_colorblind(&mut self) {
        self.board.color_blind = !self.board.color_blind;
    }
    pub fn level(&self) -> u32 {
        self.lines / 10
    }

    pub fn step_delay(&self) -> time::Duration {
        let frame_ms = 33; // Roughly 30 fps or NTS 
        let frames_per_square = match self.level() {
            0 => 48,
            1 => 43,
            2 => 38,
            3 => 33,
            4 => 28,
            5 => 23,
            6 => 18,
            7 => 13,
            8 => 8,
            9 => 6,
            10..=12 => 5,
            13..=15 => 4,
            16..=18 => 3,
            19..=28 => 2,
            _ => 1,
        };

        time::Duration::from_millis(frame_ms * frames_per_square)
    }

    pub fn new_piece(&mut self) {
        self.board.curr_piece = self.next_piece.take();
        self.next_piece = Some(rand_piece(&mut self.rng));
    }

    pub fn lost(&self) -> bool {
        self.board.cells[MAX_PIECE_SIZE - 1] != BLACK
    }

    pub fn display(&self) -> Result<(), Box<dyn Error>> {
        self.board.display(START_ROW, START_COL)?;
        self.display_info()
    }

    fn display_info(&self) -> Result<(), Box<dyn Error>> {
        // Leave two spaces to the right of the board
        let info_col = START_COL + 5;
        let info_row = START_ROW + 1;

        // let info_height = max(MAX_PIECE_SIZE, 5);
        let score_label = "Score: ";
        let lines_label = "Lines: ";
        let level_label = "Level: ";

        let num_width = 12;
        let right_width = score_label.len() + num_width;
        let mut curr_row = info_row;
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("╔═╤═"),
            Print(format!("{:═>right_width$}", "")),
            Print("═╗")
        )?;
        curr_row += 1;
        let next_cell = self
            .next_piece
            .as_ref()
            .map(|c| {
                c.cells
                    .get((curr_row - info_row) as usize - 1)
                    .unwrap_or(&BLACK)
            })
            .unwrap_or(&BLACK);
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("║"),
            RenderCell(*next_cell, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print("│ "),
            Print(lines_label),
            Print(format!(
                "{: >num_width$}",
                self.lines.to_formatted_string(&Locale::en)
            )),
            Print(" ║")
        )?;
        curr_row += 1;
        let next_cell = self
            .next_piece
            .as_ref()
            .map(|c| {
                c.cells
                    .get((curr_row - info_row) as usize - 1)
                    .unwrap_or(&BLACK)
            })
            .unwrap_or(&BLACK);
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("║"),
            RenderCell(*next_cell, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print("│ "),
            Print(format!("{: >right_width$}", "")),
            Print(" ║")
        )?;
        curr_row += 1;
        let next_cell = self
            .next_piece
            .as_ref()
            .map(|c| {
                c.cells
                    .get((curr_row - info_row) as usize - 1)
                    .unwrap_or(&BLACK)
            })
            .unwrap_or(&BLACK);
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("║"),
            RenderCell(*next_cell, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print("│ "),
            Print(level_label),
            Print(format!("{: >num_width$}", self.level())),
            Print(" ║")
        )?;
        curr_row += 1;
        let next_cell = self
            .next_piece
            .as_ref()
            .map(|c| {
                c.cells
                    .get((curr_row - info_row) as usize - 1)
                    .unwrap_or(&BLACK)
            })
            .unwrap_or(&BLACK);
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("║"),
            RenderCell(*next_cell, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print("│ "),
            Print(format!("{: >right_width$}", "")),
            Print(" ║")
        )?;
        curr_row += 1;
        let next_cell = self
            .next_piece
            .as_ref()
            .map(|c| {
                c.cells
                    .get((curr_row - info_row) as usize - 1)
                    .unwrap_or(&BLACK)
            })
            .unwrap_or(&BLACK);
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("║"),
            RenderCell(*next_cell, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print("│ "),
            Print(score_label),
            Print(format!(
                "{: >num_width$}",
                self.score.to_formatted_string(&Locale::en)
            )),
            Print(" ║")
        )?;
        curr_row += 1;
        queue!(
            stdout(),
            MoveTo(info_col, curr_row),
            SetForegroundColor(GRAY),
            Print("╚═╧═"),
            Print(format!("{:═>right_width$}", "")),
            Print("═╝")
        )?;

        // Finally, display some useful information
        curr_row += 2;

        queue!(
            stdout(),
            MoveTo(info_col + 3, curr_row),
            SetForegroundColor(GRAY),
            RenderCell(RED, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print(" + "),
            RenderCell(GREEN, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print(" = "),
            RenderCell(RED_GREEN, self.board.color_blind),
        )?;
        curr_row += 2;
        queue!(
            stdout(),
            MoveTo(info_col + 3, curr_row),
            RenderCell(RED, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print(" + "),
            RenderCell(BLUE, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print(" = "),
            RenderCell(RED_BLUE, self.board.color_blind),
        )?;
        curr_row += 2;
        queue!(
            stdout(),
            MoveTo(info_col + 3, curr_row),
            RenderCell(GREEN, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print(" + "),
            RenderCell(BLUE, self.board.color_blind),
            SetForegroundColor(GRAY),
            Print(" = "),
            RenderCell(BLUE_GREEN, self.board.color_blind),
        )?;

        // Display useful commands
        curr_row += 2;
        queue!(
            stdout(),
            SetForegroundColor(GRAY),
            MoveTo(info_col, curr_row),
            Print(format!("{:>16} : Rotate", "<Up>")),
            MoveTo(info_col, curr_row + 1),
            Print(format!("{:>16} : Drop", "<Down> or <Space>")),
            MoveTo(info_col, curr_row + 2),
            Print(format!("{:>16} : Toggle", "c")),
            MoveTo(info_col, curr_row + 3),
            Print(format!("{:>16}   Color-blind", " ")),
            MoveTo(info_col, curr_row + 4),
            Print(format!("{:>16} : Quit", "<Esc> or q")),
        )?;
        stdout().flush()?;
        Ok(())
    }
}
fn rand_piece(rng: &mut SmallRng) -> Piece {
    let size = rng.random_range(1usize..=MAX_PIECE_SIZE);
    let mut cells = vec![BLACK; size];
    for cell in cells.iter_mut() {
        *cell = PIECE_COLORS[rng.random_range(0..PIECE_COLORS.len())];
    }
    Piece { cells, pos: 0 }
}
