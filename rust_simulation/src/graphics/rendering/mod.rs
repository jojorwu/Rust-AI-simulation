use bevy::prelude::*;

pub mod entity_rendering;
pub mod map_rendering;

use entity_rendering::EntityRenderingPlugin;
use map_rendering::MapRenderingPlugin;

pub use entity_rendering::{RenderedEntity, TILE_SIZE};

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EntityRenderingPlugin, MapRenderingPlugin));
    }
}
