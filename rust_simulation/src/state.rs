use serde::{Serialize, Deserialize};
use super::player::Slot;
use super::map::Tile;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StateKey {
    pub local_view: Vec<Tile>,
    pub inventory: Vec<Option<Slot>>,
    pub held_item: Option<String>,
}
