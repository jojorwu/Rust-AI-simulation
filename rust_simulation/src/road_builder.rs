use crate::config::RoadConfig;
use crate::errors::SimulationError;
use crate::road::*;
use crate::Game;
use std::collections::HashMap;
use std::env;

pub fn generate_roads(game: &mut Game) -> Result<(), SimulationError> {
    _generate_roads_from_config(game)
}

fn _generate_roads_from_config(game: &mut Game) -> Result<(), SimulationError> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let road_config_path = format!("{manifest_dir}/data/road_config.json");
    let road_config = RoadConfig::load(&road_config_path)?;

    // This is a placeholder. In a real game, these locations might be dynamically determined.
    let mut city_locations: HashMap<String, (i32, i32)> = HashMap::new();
    city_locations.insert("CityA".to_string(), (20, 20));
    city_locations.insert("CityB".to_string(), (80, 30));
    city_locations.insert("CityC".to_string(), (30, 70));
    city_locations.insert("CityD".to_string(), (90, 85));
    city_locations.insert("Old_Mine".to_string(), (50, 90));

    for setting in road_config.road_settings {
        let start_pos = city_locations
            .get(&setting.start_point)
            .ok_or(SimulationError::CityNotFound(setting.start_point.clone()))?;
        let end_pos = city_locations
            .get(&setting.end_point)
            .ok_or(SimulationError::CityNotFound(setting.end_point.clone()))?;

        let start_point = Point {
            x: start_pos.0 as f32,
            y: start_pos.1 as f32,
        };
        let end_point = Point {
            x: end_pos.0 as f32,
            y: end_pos.1 as f32,
        };

        let road = generate_road(setting, start_point, end_point);

        for point in &road.path {
            if point.x >= 0.0
                && point.x < game.map.width as f32
                && point.y >= 0.0
                && point.y < game.map.height as f32
            {
                let x = point.x as usize;
                let y = point.y as usize;
                let tile = &mut game.map.grid[y][x];
                tile.tile_type = '=';
            }
        }
        game.road_manager.add_road(road);
    }
    Ok(())
}
