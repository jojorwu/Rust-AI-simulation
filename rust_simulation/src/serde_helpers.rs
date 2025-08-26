use crate::brain::{Goal, HighLevelState};
use serde::{de::Deserializer, ser::Serializer, Deserialize, Serialize};
use std::collections::HashMap;

pub mod q_table_map_format {
    use super::*;

    pub fn serialize<'a, S>(
        map: &'a HashMap<HighLevelState, HashMap<Goal, f64>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<_> = map.iter().collect();
        vec.serialize(serializer)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<HighLevelState, HashMap<Goal, f64>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<(HighLevelState, HashMap<Goal, f64>)> = Vec::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}
