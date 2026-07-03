use std::{
    error::Error,
    io::stdout,
    thread::sleep,
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{Event, KeyCode, poll, read},
    execute,
    style::{self, available_color_count},
    terminal::{self, enable_raw_mode},
};

use crate::game::{GAME_HEIGHT, GAME_WIDTH, Game};

mod display;
mod game;

fn main() -> Result<(), Box<dyn Error>> {
    if available_color_count() < 65535 {
        panic!("Must run in a terminal with True Color. On MacOS, try iTerm2.");
    }
    let term_size = terminal::size()?;
    if term_size.0 < GAME_WIDTH || term_size.1 < GAME_HEIGHT {
        panic!("Must run in a terminal at least {} rows high and at least {} columns wide.", GAME_WIDTH, GAME_HEIGHT);
    }
    enable_raw_mode()?;
    execute!(
        stdout(),
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(terminal::ClearType::All)
    )?;
    let _ = main_loop(Game::new())?;
    sleep(Duration::from_millis(1000));
    execute!(
        stdout(),
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    Ok(())
}

fn main_loop(game: Game) -> Result<Game, Box<dyn Error>> {
    let mut game = game;
    loop {
        // Start of frame
        let frame_start = Instant::now();
        // If there is no piece, then clear lines, wait, and create a new one
        if game.board.curr_piece.is_none() {
            sleep(game.step_delay());
            let lines_cleared = game.board.clear_lines();
            if lines_cleared > 0 {
                let mult = match lines_cleared {
                    1 => 40,
                    2 => 100,
                    3 => 400,
                    4 => 1200,
                    _ => panic!("Unsupported clear count"),
                };
                game.score += mult * (game.level() + 1);
                game.lines += lines_cleared;
                game.display()?;
                sleep(game.step_delay());
            }
            if game.lost() {
                return Ok(game);
            }
            game.new_piece();
        }
        while game.board.curr_piece.is_some() {
            game.display()?;
            // Check for input
            if poll(Duration::from_millis(10))? {
                let event = read()?;
                if event == Event::Key(KeyCode::Esc.into())
                    || event == Event::Key(KeyCode::Char('q').into())
                {
                    return Ok(game); // Quit out
                }
                if event == Event::Key(KeyCode::Up.into())
                    && let Some(piece) = game.board.curr_piece.as_mut()
                {
                    piece.rotate()
                }
                if event == Event::Key(KeyCode::Char('c').into()) {
                    game.toggle_colorblind();
                }
                if event == Event::Key(KeyCode::Down.into())
                    || event == Event::Key(KeyCode::Char(' ').into())
                {
                    while game.board.drop() {
                        game.score += 1;
                    }
                    game.board.merge_piece();
                    game.display()?;
                    break;
                }
            }
            if frame_start.elapsed() > game.step_delay() {
                if !game.board.drop() {
                    game.board.merge_piece();
                    game.display()?;
                }
                break;
            }
        }
    }
}
