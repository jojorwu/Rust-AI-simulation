// MAP SETTINGS
pub const WIDTH: u32 = 20;
pub const HEIGHT: u32 = 10;
pub const NUM_TREES: u32 = 15;
pub const NUM_STONE: u32 = 10;
pub const NUM_SULFUR: u32 = 10;
pub const NUM_IRON_ORE: u32 = 8;

// PLAYER/AI SETTINGS
pub const NUM_PLAYERS: u32 = 2;
pub const LEARNING_RATE: f64 = 0.1;
pub const DISCOUNT_FACTOR: f64 = 0.9;
pub const INITIAL_EPSILON: f64 = 1.0;
pub const MIN_EPSILON: f64 = 0.05;
pub const EPSILON_DECAY: f64 = 0.9995;

// TRAINING LOOP SETTINGS
pub const EPISODES: u32 = 2000;
pub const MAX_STEPS_PER_EPISODE: u32 = 200;
