use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode},
    execute,
    style::{Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    QueueableCommand,
};
use rand::seq::SliceRandom;
use std::{
    io::{self, Write},
    thread::sleep,
    time::Duration,
};

const BOARD_SIZE: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Board {
    tiles: Vec<u8>, // 0 represents the blank tile
}

impl Board {
    fn new() -> Self {
        let mut tiles = (1..=8).collect::<Vec<_>>();
        tiles.push(0); // Add the blank tile
        Self { tiles }
    }

    fn shuffle(&mut self) {
        let mut rng = rand::thread_rng();
        let possible_moves = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut previous_move = None;

        for _ in 0..100 {
            let mut moves = possible_moves.clone();
            if let Some(prev_move) = previous_move {
                moves.retain(|&m| m != prev_move);
            }

            let direction = moves.choose(&mut rng).unwrap();
            self.move_tile(*direction);
            previous_move = Some(*direction);
        }
    }

    fn is_solved(&self) -> bool {
        self.tiles == [1, 2, 3, 4, 5, 6, 7, 8, 0]
    }

    fn get_blank_position(&self) -> usize {
        self.tiles.iter().position(|&tile| tile == 0).unwrap()
    }

    fn move_tile(&mut self, direction: Direction) {
        let blank_pos = self.get_blank_position();

        let tile_to_move_pos = match direction {
            Direction::Up => blank_pos.checked_add(BOARD_SIZE), // Move tile UP into blank space
            Direction::Down => blank_pos.checked_sub(BOARD_SIZE), // Move tile DOWN into blank space
            Direction::Left => blank_pos.checked_add(1).filter(|pos| pos % BOARD_SIZE != 0), // Move tile LEFT into blank space
            Direction::Right => blank_pos
                .checked_sub(1)
                .filter(|pos| pos % BOARD_SIZE != BOARD_SIZE - 1), // Move tile RIGHT into blank space
        };

        if let Some(tile_to_move_pos) = tile_to_move_pos {
            if tile_to_move_pos < self.tiles.len() {
                self.tiles.swap(blank_pos, tile_to_move_pos);
            }
        }
    }

    // Additional helper methods can be added here if needed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn render_board(board: &Board) -> Result<(), io::Error> {
    // need to add a message about pressing q to quit

    let mut stdout = io::stdout();

    stdout.queue(Clear(ClearType::All))?;
    stdout.queue(Hide)?;

    for (i, &tile) in board.tiles.iter().enumerate() {
        let row = i / BOARD_SIZE;
        let col = i % BOARD_SIZE;

        stdout.queue(MoveTo(col as u16 * 4, row as u16 * 2))?;

        if tile == 0 {
            stdout.queue(Print("    "))?;
        } else {
            stdout.queue(SetBackgroundColor(crossterm::style::Color::DarkGrey))?;
            stdout.queue(SetForegroundColor(crossterm::style::Color::White))?;
            stdout.queue(Print(format!("{:^3}", tile)))?;
            stdout.queue(ResetColor)?;
        }
    }

    stdout.queue(Show)?;
    stdout.flush()?;

    Ok(())
}

fn main() -> Result<(), io::Error> {
    let mut board = Board::new();
    board.shuffle();

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen)?;

    loop {
        render_board(&board)?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Up => board.move_tile(Direction::Up),
                KeyCode::Down => board.move_tile(Direction::Down),
                KeyCode::Left => board.move_tile(Direction::Left),
                KeyCode::Right => board.move_tile(Direction::Right),
                KeyCode::Esc | KeyCode::Char('q') => break,
                _ => {}
            }

            if board.is_solved() {
                println!("You won!");
                sleep(Duration::from_secs(2));
                break;
            }
        }
    }

    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_board() {
        let board = Board::new();
        assert_eq!(board.tiles, [1, 2, 3, 4, 5, 6, 7, 8, 0]);
    }

    #[test]
    fn test_shuffle_board() {
        let mut board = Board::new();
        let initial_board = board.clone();
        board.shuffle();
        assert_ne!(board, initial_board);
    }

    #[test]
    fn test_is_solved() {
        let mut board = Board::new();
        board.shuffle();
        assert!(!board.is_solved());
        board.tiles = vec![1, 2, 3, 4, 5, 6, 7, 8, 0];
        assert!(board.is_solved());
    }

    #[test]
    fn test_get_blank_position() {
        let board = Board::new();
        assert_eq!(board.get_blank_position(), 8);
    }

    #[test]
    fn test_move_tiles() {
        let mut board = Board::new();
        board.move_tile(Direction::Up);
        assert_eq!(board.tiles, [1, 2, 3, 4, 5, 6, 7, 8, 0]);
        board.move_tile(Direction::Left);
        assert_eq!(board.tiles, [1, 2, 3, 4, 5, 6, 7, 8, 0]);
        board.move_tile(Direction::Down);
        assert_eq!(board.tiles, [1, 2, 3, 4, 5, 0, 7, 8, 6]);
        board.move_tile(Direction::Right);
        assert_eq!(board.tiles, [1, 2, 3, 4, 0, 5, 7, 8, 6]);
        board.move_tile(Direction::Right);
        assert_eq!(board.tiles, [1, 2, 3, 0, 4, 5, 7, 8, 6]);
        board.move_tile(Direction::Down);
        assert_eq!(board.tiles, [0, 2, 3, 1, 4, 5, 7, 8, 6]);
        board.move_tile(Direction::Up);
        assert_eq!(board.tiles, [1, 2, 3, 0, 4, 5, 7, 8, 6]);
    }
}
