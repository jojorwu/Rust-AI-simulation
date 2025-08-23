use bevy::prelude::*;

mod rendering;
mod ui;

use rendering::RenderingPlugin;
use ui::UiPlugin;

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera)
            .add_plugins((RenderingPlugin, UiPlugin));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}
