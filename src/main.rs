//! # Game of Life
//!
//! A simple simulation of Conway's Game of Life in
//! [raylib-rs](https://github.com/deltaphc/raylib-rs).
//!
//! Controls:
//! - *Edit Mode:* Left-click to create a tile. Right-click to remove a tile.
//! `x` to clear all tiles. `SPACE` to enter *Automata Mode*.
//! - *Automata Mode:* `SPACE` to re-enter *Edit Mode*. The program should also
//! automatically return to *Edit Mode* when there are either no living elements
//! or no moving elements on screen.
//!
//! **Author:** Jude Muriithi (GitHub:
//! [jkmuriithi](https://github.com/jkmuriithi))
//!

// Hide debug console on Windows when necessary
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::time::Instant;

use raylib::consts::MouseButton::*;
use raylib::prelude::*;

// Game configuration constants
const TICKS_PER_SECOND: u128 = 10;
const GRID_SCALE: i32 = 20;
const GRID_WIDTH: i32 = 40;
const GRID_HEIGHT: i32 = 30;

const GRID_COLOR: Color = Color::LIGHTGRAY;
const LIVE_COLOR: Color = Color::BLACK;
const DEAD_COLOR: Color = Color::WHITE;

// Helper constants
const NANOS_PER_TICK: u128 = 1_000_000_000 / TICKS_PER_SECOND;
const SCREEN_WIDTH: i32 = GRID_WIDTH * GRID_SCALE;
const SCREEN_HEIGHT: i32 = GRID_HEIGHT * GRID_SCALE;
const U_GRID_WIDTH: usize = GRID_WIDTH as usize;
const U_GRID_HEIGHT: usize = GRID_HEIGHT as usize;

enum GameState {
    Editing,
    Running,
}

fn main() {
    let (mut rl, thread) =
        raylib::init().size(SCREEN_WIDTH, SCREEN_HEIGHT).vsync().build();

    let start = Instant::now();
    let mut current_tick: u128 = 0;
    let mut state = GameState::Editing;

    let mut grid = new_grid();
    while !rl.window_should_close() {
        match state {
            GameState::Editing => {
                // Use mouse to set squares
                if rl.is_mouse_button_down(MOUSE_LEFT_BUTTON) {
                    let (x, y) = px_to_square(rl.get_mouse_position());
                    grid[y][x] = true;
                } else if rl.is_mouse_button_down(MOUSE_RIGHT_BUTTON) {
                    let (x, y) = px_to_square(rl.get_mouse_position());
                    grid[y][x] = false;
                }

                match rl.get_key_pressed() {
                    Some(KeyboardKey::KEY_SPACE) => state = GameState::Running,
                    Some(KeyboardKey::KEY_X) => grid = new_grid(),
                    _ => (),
                }
            }
            GameState::Running => {
                let tick = start.elapsed().as_nanos() / NANOS_PER_TICK;

                if tick > current_tick {
                    current_tick = tick;

                    let mut next_grid = new_grid();
                    let mut num_alive = 0;
                    let mut still_frame = true;

                    for x in 0..U_GRID_WIDTH {
                        for y in 0..U_GRID_HEIGHT {
                            let neighbor_count = living_neighbors(&grid, x, y);

                            if grid[y][x] {
                                if neighbor_count == 2 || neighbor_count == 3 {
                                    next_grid[y][x] = true;
                                    num_alive += 1;
                                } else {
                                    still_frame = false;
                                }
                            } else if neighbor_count == 3 {
                                next_grid[y][x] = true;
                                num_alive += 1;
                                still_frame = false;
                            }
                        }
                    }

                    if still_frame || num_alive == 0 {
                        state = GameState::Editing;
                    }

                    grid = next_grid;
                }

                if let Some(KeyboardKey::KEY_SPACE) = rl.get_key_pressed() {
                    state = GameState::Editing;
                }
            }
        }

        let mut d = rl.begin_drawing(&thread);

        // Draw squares
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let alive = grid[y as usize][x as usize];
                d.draw_rectangle(
                    x * GRID_SCALE,
                    y * GRID_SCALE,
                    GRID_SCALE,
                    GRID_SCALE,
                    if alive { LIVE_COLOR } else { DEAD_COLOR },
                );
            }
        }

        // Draw grid lines
        for i in 1..GRID_WIDTH {
            let x_px = i * GRID_SCALE;
            d.draw_line(x_px, 0, x_px, SCREEN_HEIGHT, GRID_COLOR);
        }
        for j in 1..GRID_HEIGHT {
            let y_px = j * GRID_SCALE;
            d.draw_line(0, y_px, SCREEN_WIDTH, y_px, GRID_COLOR);
        }

        if let GameState::Editing = state {
            d.draw_text("Edit Mode", 15, 15, 25, Color::DARKBLUE);
        }
    }
}

/// Transforms vector pixel coordinates into indices on the automata grid.
fn px_to_square(Vector2 { x, y }: Vector2) -> (usize, usize) {
    let x = (x as i32).clamp(0, SCREEN_WIDTH - GRID_SCALE);
    let y = (y as i32).clamp(0, SCREEN_HEIGHT - GRID_SCALE);
    let x = x - (x % GRID_SCALE);
    let y = y - (y % GRID_SCALE);

    let x = x / GRID_SCALE;
    let y = y / GRID_SCALE;
    (x as usize, y as usize)
}

/// Returns a new automata grid filled with dead squares.
fn new_grid() -> Vec<Vec<bool>> {
    vec![vec![false; U_GRID_WIDTH]; U_GRID_HEIGHT]
}

/// Returns the number of living neighbors which square `(x, y)` has in `grid`.
fn living_neighbors(grid: &[Vec<bool>], x: usize, y: usize) -> u8 {
    let left = if x == 0 { x } else { x - 1 };
    let right = if x == U_GRID_WIDTH - 1 { x } else { x + 1 };
    let above = if y == 0 { y } else { y - 1 };
    let below = if y == U_GRID_HEIGHT - 1 { y } else { y + 1 };

    [
        grid[y][left],
        grid[y][right],
        grid[above][x],
        grid[below][x],
        grid[above][left],
        grid[above][right],
        grid[below][left],
        grid[below][right],
    ]
    .into_iter()
    .map(|b| b as u8)
    .sum()
}
