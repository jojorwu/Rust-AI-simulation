use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    Move(Direction),
    Gather,
    Craft(String),
    Equip(String),
    Place(String),
    Smelt,
    Build(String),
    Open,
    Close,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub fn get_all_actions() -> Vec<Action> {
    vec![
        Action::Move(Direction::Up),
        Action::Move(Direction::Down),
        Action::Move(Direction::Left),
        Action::Move(Direction::Right),
        Action::Gather,
        Action::Craft("stone_axe".to_string()),
        Action::Craft("stone_pickaxe".to_string()),
        Action::Craft("furnace".to_string()),
        Action::Craft("metal_pickaxe".to_string()),
        Action::Equip("stone_axe".to_string()),
        Action::Equip("stone_pickaxe".to_string()),
        Action::Equip("metal_pickaxe".to_string()),
        Action::Place("furnace".to_string()),
        Action::Smelt,
        Action::Build("foundation".to_string()),
        Action::Build("wall".to_string()),
        Action::Build("doorway".to_string()),
        Action::Open,
        Action::Close,
    ]
}
