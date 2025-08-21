//! This module contains the `Renderer` struct, which is responsible for all
//! console output for the simulation.

use crate::components::{BrainComponent, Inventory};
use crate::config::{EPISODES, MAX_STEPS_PER_EPISODE};
use crate::map::{Map, TileState};
use crate::player::Player;
use crate::Game;
use bevy_ecs::prelude::*;

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render(&self, game: &Game) {
        let world = &game.parallel_state.world;
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!(
            "--- Episode: {}/{} | Step: {}/{} ---",
            game.tick_count / MAX_STEPS_PER_EPISODE + 1,
            EPISODES,
            game.tick_count % MAX_STEPS_PER_EPISODE + 1,
            MAX_STEPS_PER_EPISODE
        );
        let time_of_day = if game.is_day() { "Day" } else { "Night" };
        println!("Time: {time_of_day}");

        self.display_player_map(&game.parallel_state.map, world);
        self.display_observer_map(&game.parallel_state.map, world);
        self.display_debug_info(world);
    }

    fn display_player_map(&self, map: &Map, world: &World) {
        let mut player_query = world.query::<&Player>();
        let player = player_query.iter(world).next();

        if let Some(player) = player {
            let mental_map = &player.mental_map;

            for y in 0..map.height {
                for x in 0..map.width {
                    let tile_state = mental_map.grid[y as usize][x as usize];

                    let (chunk_x, chunk_y) = map.get_chunk_coords(x, y);
                    let (tile_x, tile_y) = map.get_tile_coords_in_chunk(x, y);
                    let chunk = map.chunks[chunk_y as usize][chunk_x as usize].lock().unwrap();

                    match tile_state {
                        TileState::Unseen => print!("  "), // Two spaces for alignment
                        TileState::Explored => {
                            print!(
                                "\x1b[90m{} \x1b[0m",
                                chunk.tiles[tile_y as usize][tile_x as usize].tile_type
                            ); // Dim gray color
                        }
                        TileState::Visible => {
                            let entity_on_tile =
                                chunk.spatial_map.get(&(x, y)).and_then(|v| v.first());

                            if let Some(&entity) = entity_on_tile {
                                if world.get::<Player>(entity).is_some() {
                                    print!("\x1b[91mP \x1b[0m"); // Bright Red 'P'
                                } else {
                                    print!("\x1b[33mE \x1b[0m"); // Yellow 'E'
                                }
                            } else {
                                print!(
                                    "\x1b[97m{} \x1b[0m",
                                    chunk.tiles[tile_y as usize][tile_x as usize].tile_type
                                ); // Bright White
                            }
                        }
                    }
                }
                println!();
            }
        }
    }

    fn display_observer_map(&self, map: &Map, world: &World) {
        println!("\n--- Observer Map ---");
        for y in 0..map.height {
            for x in 0..map.width {
                let (chunk_x, chunk_y) = map.get_chunk_coords(x, y);
                let (tile_x, tile_y) = map.get_tile_coords_in_chunk(x, y);
                let chunk = map.chunks[chunk_y as usize][chunk_x as usize].lock().unwrap();
                let entity_on_tile = chunk.spatial_map.get(&(x, y)).and_then(|v| v.first());

                if let Some(&entity) = entity_on_tile {
                    if world.get::<Player>(entity).is_some() {
                        print!("\x1b[91mP \x1b[0m"); // Bright Red 'P'
                    } else {
                        print!("\x1b[33mE \x1b[0m"); // Yellow 'E'
                    }
                } else {
                    let tile_char = chunk.tiles[tile_y as usize][tile_x as usize].tile_type;
                    match tile_char {
                        '.' => print!("\x1b[32m. \x1b[0m"), // Green
                        'f' => print!("\x1b[93mf \x1b[0m"), // Bright Yellow
                        'M' => print!("\x1b[97mM \x1b[0m"), // Bright White
                        'T' => print!("\x1b[32m T\x1b[0m"), // Dark Green
                        '~' => print!("\x1b[34m~ \x1b[0m"), // Blue
                        '#' => print!("\x1b[90m# \x1b[0m"), // Dim White
                        'O' => print!("\x1b[36mO \x1b[0m"), // Cyan
                        _ => print!("{tile_char} "),
                    }
                }
            }
            println!();
        }
    }

    fn display_debug_info(&self, world: &World) {
        let mut query = world.query::<(Entity, &BrainComponent, &Inventory)>();
        if let Some((_, brain_component, inventory)) = query.iter(world).next() {
            println!("Agent 0 Goal: {:?}", brain_component.current_goal);
            println!("Agent 0 Inventory: {:?}", inventory.items);
        }
    }

    pub fn print_intro(&self) {
        println!("--- Starting Rust Training Simulation ---");
    }

    pub fn print_outro(&self) {
        println!("--- Training Finished ---");
    }
}
