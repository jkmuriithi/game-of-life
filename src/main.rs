use std::time::Instant;

use raylib::prelude::*;

use grid::{Grid, GameState};

const TICKS_PER_SECOND: u128 = 10;
const NANOS_PER_TICK: u128 = 1_000_000_000 / TICKS_PER_SECOND;

const GRID_SCALE: i32 = 40;
const GRID_WIDTH: i32 = 20;
const GRID_HEIGHT: i32 = 15;
const U_GRID_WIDTH: usize = GRID_WIDTH as usize;
const U_GRID_HEIGHT: usize = GRID_HEIGHT as usize;

const SCREEN_WIDTH: i32 = GRID_WIDTH * GRID_SCALE;
const SCREEN_HEIGHT: i32 = GRID_HEIGHT * GRID_SCALE;

const GRID_COLOR: Color = Color::LIGHTGRAY;
const LIVE_COLOR: Color = Color::BLACK;
const DEAD_COLOR: Color = Color::WHITE;

/// Transforms vector pixel coordinates into indices on the automata grid.
fn px_to_square(Vector2 { x, y }: Vector2) -> (i32, i32) {
    let x = (x as i32).clamp(0, SCREEN_WIDTH - GRID_SCALE);
    let y = (y as i32).clamp(0, SCREEN_HEIGHT - GRID_SCALE);
    let x = x - (x % GRID_SCALE);
    let y = y - (y % GRID_SCALE);

    let x = x / GRID_SCALE;
    let y = y / GRID_SCALE;
    (x, y)
}

fn main() {
    let (mut rl, thread) =
        raylib::init().size(SCREEN_WIDTH, SCREEN_HEIGHT).build();

    let start = Instant::now();
    let mut current_tick: u128 = 0;

    let mut grid = Grid::new();

    let mut state = GameState::CREATING;

    while !rl.window_should_close() {
        match state {
            GameState::CREATING => {
                // Use mouse to set squares
                if rl.is_mouse_button_down(MouseButton::MOUSE_LEFT_BUTTON) {
                    let (x, y) = px_to_square(rl.get_mouse_position());
                    grid.set_true(x as usize, y as usize);
                    println!("Neighbors: {}", grid.live_neighbors(x as usize, y as usize))
                }
                if rl.is_mouse_button_down(MouseButton::MOUSE_RIGHT_BUTTON) {
                    let (x, y) = px_to_square(rl.get_mouse_position());
                    grid.set_false(x as usize, y as usize);
                }

                // Start automata with space key
                if let Some(KeyboardKey::KEY_SPACE) = rl.get_key_pressed() {
                    state = GameState::RUNNING;
                }
            }
            GameState::RUNNING => {
                let tick = start.elapsed().as_nanos() / NANOS_PER_TICK;

                if tick > current_tick {
                    current_tick = tick;

                    let mut next = Grid::new();

                    for x in 0..GRID_WIDTH {
                        for y in 0..GRID_HEIGHT {
                            let x = x as usize;
                            let y = y as usize;

                            let neighbor_count = grid.live_neighbors(x, y);

                            if grid.is_alive(x, y) {
                                if neighbor_count == 2 || neighbor_count == 3 {
                                    next.set_true(x, y);
                                }
                            } else {
                                if neighbor_count == 3 {
                                    next.set_true(x, y);
                                }
                            }
                        }
                    }

                    grid = next;
                }

                if grid.num_alive == 0 {
                    state = GameState::CREATING;
                }
                if let Some(KeyboardKey::KEY_SPACE) = rl.get_key_pressed() {
                    state = GameState::CREATING;
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);

        // Squares
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                if grid.is_alive(x as usize, y as usize) {
                    d.draw_rectangle(
                        x * GRID_SCALE,
                        y * GRID_SCALE,
                        GRID_SCALE,
                        GRID_SCALE,
                        LIVE_COLOR,
                    );
                } else {
                    d.draw_rectangle(
                        x * GRID_SCALE,
                        y * GRID_SCALE,
                        GRID_SCALE,
                        GRID_SCALE,
                        DEAD_COLOR,
                    );
                }
            }
        }

        // Grid lines
        for i in 1..GRID_WIDTH {
            let x_px = i * GRID_SCALE;
            d.draw_line(x_px, 0, x_px, SCREEN_HEIGHT, GRID_COLOR);
        }
        for j in 1..GRID_HEIGHT {
            let y_px = j * GRID_SCALE;
            d.draw_line(0, y_px, SCREEN_WIDTH, y_px, GRID_COLOR);
        }

        // Create mode tag
        if let GameState::CREATING = state {
            d.draw_text("Create Mode", 5, 5, 20, Color::GOLD);
        }
    }
}

mod grid {
    use crate::{GRID_HEIGHT as GH, GRID_WIDTH as GW};

    const GRID_WIDTH: usize = GW as usize;
    const GRID_HEIGHT: usize = GH as usize;

    pub enum GameState {
        CREATING,
        RUNNING,
    }

    pub struct Grid {
        squares: Vec<Vec<bool>>,
        pub num_alive: u128,
    }

    impl Grid {
        pub fn new() -> Self {
            Grid {
                squares: vec![vec![false; GRID_WIDTH]; GRID_HEIGHT],
                num_alive: 0,
            }
        }

        pub fn set_true(&mut self, x: usize, y: usize) {
            if !self.squares[y][x] {
                self.squares[y][x] = true;
                self.num_alive += 1;
            }
        }

        pub fn set_false(&mut self, x: usize, y: usize) {
            if self.squares[y][x] {
                self.squares[y][x] = false;
                self.num_alive -= 1;
            }
        }

        pub fn is_alive(&self, x: usize, y: usize) -> bool {
            self.squares[y][x]
        }

        pub fn live_neighbors(&self, x: usize, y: usize) -> u8 {
            let left = if x == 0 { x } else { x - 1 };
            let right = if x == GRID_WIDTH - 1 { x } else { x + 1 };
            let above = if y == 0 { y } else { y - 1 };
            let below = if y == GRID_HEIGHT - 1 { y } else { y + 1 };

            [
                self.squares[y][left],
                self.squares[y][right],
                self.squares[above][x],
                self.squares[below][x],
                self.squares[above][left],
                self.squares[above][right],
                self.squares[below][left],
                self.squares[below][right],
            ]
            .into_iter()
            .map(|b| b as u8)
            .sum()
        }
    }
}
