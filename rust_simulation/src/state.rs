use serde::{Serialize, Deserialize};
use super::player::Slot;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct StateKey {
    pub local_view: Vec<char>,
    pub inventory: Vec<Option<Slot>>,
    pub held_item: Option<String>,
}
