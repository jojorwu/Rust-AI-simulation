use bevy::{prelude::*, window::PrimaryWindow};
use bevy::input::ButtonInput;
use crate::components::{BrainComponent, Inventory};
use crate::Game;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePlayerInventory>()
            .add_systems(Startup, (setup_status_ui, setup_inventory_panel))
            .add_systems(Update, (update_status_ui, player_click_system, update_inventory_panel));
    }
}

// --- Resources ---
#[derive(Resource, Default)]
pub(super) struct ActivePlayerInventory(pub(super) Option<Entity>);

// --- Components ---
#[derive(Component)]
pub(super) struct PlayerStatusText;

#[derive(Component)]
pub(super) struct InventoryPanel;

#[derive(Component)]
pub(super) struct InventoryText;

// --- Systems ---
pub(super) fn setup_status_ui(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Player Goal: ",
            TextStyle { font_size: 20.0, color: Color::WHITE, ..default() },
        ).with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        PlayerStatusText,
    ));
}

pub(super) fn setup_inventory_panel(mut commands: Commands) {
    commands.spawn((
        NodeBundle {
            style: Style {
                width: Val::Px(200.0),
                min_height: Val::Px(100.0),
                position_type: PositionType::Absolute,
                right: Val::Px(10.0),
                top: Val::Px(10.0),
                border: UiRect::all(Val::Px(2.0)),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            background_color: Color::rgb(0.1, 0.1, 0.1).into(),
            border_color: Color::WHITE.into(),
            visibility: Visibility::Hidden,
            ..default()
        },
        InventoryPanel,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section("Inventory:", TextStyle { font_size: 18.0, color: Color::WHITE, ..default() }));
        parent.spawn((
            TextBundle::from_section("", TextStyle { font_size: 16.0, color: Color::WHITE, ..default() })
                .with_style(Style { margin: UiRect::top(Val::Px(5.0)), ..default() }),
            InventoryText,
        ));
    });
}

pub(super) fn update_status_ui(mut game: ResMut<Game>, mut query: Query<&mut Text, With<PlayerStatusText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        let mut player_query = game.world.query_filtered::<Entity, With<crate::player::Player>>();
        let player_entity = player_query.iter(&game.world).next();
        if let Some(player_entity) = player_entity {
            if let Some(brain) = game.world.get::<BrainComponent>(player_entity) {
                text.sections[0].value = format!("Player Goal: {:?}", brain.current_goal);
            }
        }
    }
}

pub(super) fn player_click_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<(&Transform, &super::rendering::RenderedEntity)>,
    mut active_inventory: ResMut<ActivePlayerInventory>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera_query.single();
        let window = window_query.single();

        if let Some(cursor_pos) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
                let mut clicked_on_player = None;
                for (player_transform, rendered_entity) in player_query.iter() {
                    let player_pos = player_transform.translation.truncate();
                    let distance = world_pos.distance(player_pos);
                    if distance < super::rendering::TILE_SIZE / 2.0 {
                        clicked_on_player = Some(rendered_entity.0);
                        break;
                    }
                }

                if let Some(clicked_player) = clicked_on_player {
                    if active_inventory.0 == Some(clicked_player) {
                         active_inventory.0 = None; // Toggle off
                    } else {
                        active_inventory.0 = Some(clicked_player); // Toggle on
                    }
                } else {
                    active_inventory.0 = None; // Clicked outside
                }
            }
        }
    }
}

pub(super) fn update_inventory_panel(
    active_inventory: Res<ActivePlayerInventory>,
    game: Res<Game>,
    mut panel_query: Query<&mut Visibility, With<InventoryPanel>>,
    mut text_query: Query<&mut Text, With<InventoryText>>,
) {
    let mut panel_visibility = panel_query.single_mut();
    if let Some(player_entity) = active_inventory.0 {
        *panel_visibility = Visibility::Inherited;
        if let Some(inventory) = game.world.get::<Inventory>(player_entity) {
            let mut text = text_query.single_mut();
            let items_str = inventory.items.iter()
                .map(|(item_id, quantity)| format!("- {}: {}", item_id, quantity))
                .collect::<Vec<_>>()
                .join("\n");
            text.sections[0].value = if items_str.is_empty() { "Empty".to_string() } else { items_str };
        }
    } else {
        *panel_visibility = Visibility::Hidden;
    }
}
