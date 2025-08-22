//! This module contains the `Renderer` struct, which is responsible for all
//! console output for the simulation.

use crate::components::{BrainComponent, Inventory};
use crate::config::{EPISODES, MAX_STEPS_PER_EPISODE};
use crate::map::Map;
use crate::player::Player;
use crate::Game;
use bevy_ecs::prelude::*;

pub struct Renderer;

impl Renderer {
    pub fn new() -> Self {
        Renderer
    }

    pub fn render(&self, game: &mut Game) {
        let tick_count = game.tick_count();
        let is_day = game.is_day();
        let world = &mut game.world;
        print!("\x1B[2J\x1B[1;1H"); // Clear screen
        println!(
            "--- Episode: {}/{} | Step: {}/{} ---",
            tick_count / MAX_STEPS_PER_EPISODE + 1,
            EPISODES,
            tick_count % MAX_STEPS_PER_EPISODE + 1,
            MAX_STEPS_PER_EPISODE
        );
        let time_of_day = if is_day { "Day" } else { "Night" };
        println!("Time: {time_of_day}");

        let map = world.get_resource::<Map>().unwrap();
        self.display_player_map(map, world);
        self.display_observer_map(map, world);
        self.display_debug_info(world);
    }

    fn display_player_map(&self, map: &Map, world: &World) {
        println!("\n--- Player Map ---");
        // Simplified to show the whole map
        self.display_observer_map(map, world);
    }

    fn display_observer_map(&self, map: &Map, world: &World) {
        println!("\n--- Observer Map ---");
        for y in 0..map.height {
            for x in 0..map.width {
                let entity_on_tile = map.get_entities_at(x, y).and_then(|v| v.first().copied());

                if let Some(entity) = entity_on_tile {
                    if world.get::<Player>(entity).is_some() {
                        print!("\x1b[91mP \x1b[0m"); // Bright Red 'P'
                    } else {
                        print!("\x1b[33mE \x1b[0m"); // Yellow 'E'
                    }
                } else {
                    if let Some(tile) = map.get_tile(x, y) {
                        let tile_char = tile.tile_type;
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
                    } else {
                        print!("  ");
                    }
                }
            }
            println!();
        }
    }

    fn display_debug_info(&self, world: &mut World) {
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
