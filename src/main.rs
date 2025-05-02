//! Game of Life simulation using minifb for windowed rendering.
//! Features: random initialization, tick-based updates, restart & speed controls.

use minifb::{Key, Window, WindowOptions}; // Window management & input
use rand::Rng;                            // Random number generation for initial state
use std::{thread, time::Duration};        // Timing control for frame delays

/// Represents the Game of Life grid and its state.
struct Universe {
    width: u32,   // number of columns
    height: u32,   // number of rows
    cells: Vec<u8>, // flat vector: 1 = alive, 0 = dead
}

impl Universe {
    /// Creates a new universe with random alive/dead cells.
    fn new(width: u32, height: u32) -> Universe {
        let mut rng = rand::rng();
        // Generate width*height cells, randomly alive (1) or dead (0)
        let cells = (0..width * height)
            .map(|_| if rng.random_bool(0.5) { 1 } else { 0 })
            .collect();
        Universe { width, height, cells }
    }

    /// Exposes the internal cell array for rendering.
    fn cells(&self) -> &[u8] {
        &self.cells
    }

    /// Advances the universe by one tick, applying Conway's rules.
    fn tick(&mut self) {
        let mut next = self.cells.clone();
        // Iterate each cell position
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = (row * self.width + col) as usize;
                let live_neighbors = self.live_neighbor_count(row, col);
                // Apply rules of Life
                next[idx] = match (self.cells[idx], live_neighbors) {
                    (1, x) if x < 2 => 0,                // underpopulation
                    (1, 2) | (1, 3) => 1,                // lives on
                    (1, x) if x > 3 => 0,                // overpopulation
                    (0, 3) => 1,                // reproduction
                    (c, _) => c,                // otherwise stays the same
                };
            }
        }
        self.cells = next;
    }

    /// Counts alive neighbors around a given cell (with wrap-around edges).
    fn live_neighbor_count(&self, row: u32, col: u32) -> u8 {
        let deltas = [self.height - 1, 0, 1];
        deltas.iter()
            .flat_map(|&dr| deltas.iter().map(move |&dc| (dr, dc)))
            .filter(|&(dr, dc)| !(dr == 0 && dc == 0))
            .map(|(dr, dc)| {
                let r = (row + dr) % self.height;
                let c = (col + dc) % self.width;
                self.cells[(r * self.width + c) as usize]
            })
            .sum()
    }
}

fn main() {
    // --- Configuration ---
    let width = 128;    // grid width
    let height = 128;    // grid height
    let scale = 4;      // pixel size of each cell
    let mut delay = 50;  // milliseconds between frames

    // Calculate total pixel buffer size
    let buffer_size = (width * height * scale * scale) as usize;

    // --- Window Setup ---
    let mut window = Window::new(
        "Game of Life — minifb",
        (width * scale) as usize,
        (height * scale) as usize,
        WindowOptions {
            resize: false,
            ..Default::default()
        },
    ).expect("Unable to open minifb window");

    // Initialize universe state and pixel buffer
    let mut universe = Universe::new(width, height);
    let mut buffer: Vec<u32> = vec![0; buffer_size];

    // --- Main Loop ---
    while window.is_open() && !window.is_key_down(Key::Escape) {
        // -- Controls --
        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            // Restart simulation
            universe = Universe::new(width, height);
            println!("🔄 Restarted universe");
        }
        if window.is_key_pressed(Key::Up, minifb::KeyRepeat::No) {
            // Speed up (lower delay)
            if delay > 5 {
                delay -= 5;
                println!("⚡️ Speed increased, delay = {}ms", delay);
            }
        }
        if window.is_key_pressed(Key::Down, minifb::KeyRepeat::No) {
            // Slow down (higher delay)
            delay += 5;
            println!("🐢 Speed decreased, delay = {}ms", delay);
        }

        // -- Rendering --
        // Map each cell to a block of pixels in the buffer
        for row in 0..height {
            for col in 0..width {
                let idx = (row * width + col) as usize;
                let alive = universe.cells()[idx] == 1;
                let color = if alive { 0xFFFFFF } else { 0x000000 };
                // Fill a scale×scale square for each cell
                for dy in 0..scale {
                    for dx in 0..scale {
                        let x = (col * scale + dx) as usize;
                        let y = (row * scale + dy) as usize;
                        buffer[y * (width as usize * scale as usize) + x] = color;
                    }
                }
            }
        }

        // Display buffer and step simulation
        window
            .update_with_buffer(&buffer, (width * scale) as usize, (height * scale) as usize)
            .unwrap();
        universe.tick();

        // Pause to control speed
        thread::sleep(Duration::from_millis(delay));
    }
}