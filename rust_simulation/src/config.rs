use serde::Deserialize;
use std::fs;

// MAP SETTINGS
pub const WIDTH: u32 = 100;
pub const HEIGHT: u32 = 100;

// PLAYER/AI SETTINGS
pub const NUM_PLAYERS: u32 = 2;

// TRAINING LOOP SETTINGS
pub const EPISODES: u32 = 1;
pub const MAX_STEPS_PER_EPISODE: u32 = 20;

// DAY/NIGHT CYCLE SETTINGS
pub const DAY_LENGTH: u32 = 100;
pub const NIGHT_LENGTH: u32 = 50;

// AI Configuration
pub const OPPORTUNISTIC_COMMITMENT_THRESHOLD: u32 = 5;
pub const VALUABLE_RESOURCES: &[&str] = &["iron_ore"];
pub const DEFENSE_RADIUS: u32 = 10;
pub const CRITICAL_HEALTH_RATIO: f32 = 0.25;
pub const STANDARD_HEALTH_RATIO: f32 = 0.5;

#[derive(Deserialize, Debug, Clone)]
pub struct RoadSetting {
    pub name: String,
    pub start_point: String,
    pub end_point: String,
    pub width: u32,
    pub curvature: f32,
    pub material: String,
    pub terrain_following_strength: f32,
}

#[derive(Deserialize, Debug)]
pub struct RoadConfig {
    pub road_count: u32,
    pub allow_intersections: bool,
    pub road_settings: Vec<RoadSetting>,
}

impl RoadConfig {
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let data = fs::read_to_string(path)?;
        let config: RoadConfig = serde_json::from_str(&data)?;
        Ok(config)
    }
}
