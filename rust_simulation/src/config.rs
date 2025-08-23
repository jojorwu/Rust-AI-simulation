use crate::errors::SimulationError;
use bevy::prelude::Reflect;
use bevy_ecs::prelude::Resource;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Resource, Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct Config {
    pub map_settings: MapSettings,
    pub player_settings: PlayerSettings,
    pub training_settings: TrainingSettings,
    pub day_night_cycle: DayNightCycle,
    pub ai: Ai,
    pub performance: PerformanceSettings,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct MapSettings {
    pub width: u32,
    pub height: u32,
    pub seed: Option<u32>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct PlayerSettings {
    pub num_players: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct TrainingSettings {
    pub episodes: u32,
    pub max_steps_per_episode: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct DayNightCycle {
    pub day_length: u32,
    pub night_length: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct Ai {
    pub opportunistic_commitment_threshold: u32,
    pub valuable_resources: Vec<String>,
    pub defense_radius: u32,
    pub critical_health_ratio: f32,
    pub standard_health_ratio: f32,
    pub q_learning: QLearning,
    pub goals: Goals,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct QLearning {
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub epsilon: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct Goals {
    pub reward: f64,
    pub penalty: f64,
    pub build_bonus: f64,
    pub gather_threshold: u32,
    pub commitment_ticks: u32,
    pub threat_commitment_ticks: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, Reflect)]
pub struct PerformanceSettings {
    pub processor_cores: u32,
    pub ram_limit_gb: u32,
    pub enable_ram_limit: bool,
}

impl Config {
    pub fn load(path: &str) -> Result<Self, SimulationError> {
        let data = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&data)?;
        Ok(config)
    }
}


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
    pub fn load(path: &str) -> Result<Self, SimulationError> {
        let data = fs::read_to_string(path)?;
        let config: RoadConfig = serde_json::from_str(&data)?;
        Ok(config)
    }
}
