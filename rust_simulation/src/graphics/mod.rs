use bevy::{prelude::*, window::PrimaryWindow};
use bevy::input::ButtonInput;
use crate::map::{Map, Tile};
use crate::player::Player;
use crate::components::{Position, BrainComponent, Inventory};
use crate::Game;

const TILE_SIZE: f32 = 32.0;

// --- Plugin ---

pub struct GraphicsPlugin;

impl Plugin for GraphicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActivePlayerInventory>()
            .add_systems(Startup, (
                setup_camera,
                setup_map_and_entities,
                setup_status_ui,
                setup_inventory_panel,
            ))
            .add_systems(Update, (
                update_entity_positions,
                update_status_ui,
                player_click_system,
                update_inventory_panel,
            ));
    }
}

// --- Resources ---

#[derive(Resource, Default)]
struct ActivePlayerInventory(Option<Entity>);

// --- Components ---

#[derive(Component)]
struct RenderedEntity(Entity);

#[derive(Component)]
struct PlayerStatusText;

#[derive(Component)]
struct InventoryPanel;

#[derive(Component)]
struct InventoryText;


// --- Systems ---

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_map_and_entities(mut commands: Commands, mut game: ResMut<Game>) {
    let map = game.world.get_resource::<Map>().unwrap();

    // Spawn map tiles
    for y in 0..map.height {
        for x in 0..map.width {
            if let Some(tile) = map.get_tile(x, y) {
                commands.spawn(SpriteBundle {
                    sprite: Sprite {
                        color: tile_type_to_color(&tile),
                        custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                        ..default()
                    },
                    transform: Transform::from_xyz(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 0.0),
                    ..default()
                });
            }
        }
    }

    // Spawn player sprite
    let mut player_query = game.world.query_filtered::<Entity, With<Player>>();
    for entity in player_query.iter(&game.world) {
        let position = game.world.get::<Position>(entity).unwrap();
        let spawned_entity = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::RED,
                    custom_size: Some(Vec2::new(TILE_SIZE, TILE_SIZE)),
                    ..default()
                },
                transform: Transform::from_xyz(position.x as f32 * TILE_SIZE, position.y as f32 * TILE_SIZE, 1.0),
                ..default()
            },
        )).id();
        commands.entity(spawned_entity).insert(RenderedEntity(entity));
    }
}

fn setup_status_ui(mut commands: Commands) {
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

fn setup_inventory_panel(mut commands: Commands) {
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

fn update_entity_positions(game: Res<Game>, mut query: Query<(&mut Transform, &RenderedEntity)>) {
    for (mut transform, rendered_entity) in query.iter_mut() {
        if let Some(position) = game.world.get::<Position>(rendered_entity.0) {
            transform.translation.x = position.x as f32 * TILE_SIZE;
            transform.translation.y = position.y as f32 * TILE_SIZE;
        }
    }
}

fn update_status_ui(mut game: ResMut<Game>, mut query: Query<&mut Text, With<PlayerStatusText>>) {
    if let Ok(mut text) = query.get_single_mut() {
        let mut player_query = game.world.query_filtered::<Entity, With<Player>>();
        let player_entity = player_query.iter(&game.world).next();
        if let Some(player_entity) = player_entity {
            if let Some(brain) = game.world.get::<BrainComponent>(player_entity) {
                text.sections[0].value = format!("Player Goal: {:?}", brain.current_goal);
            }
        }
    }
}

fn player_click_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    player_query: Query<(&Transform, &RenderedEntity)>,
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
                    if distance < TILE_SIZE / 2.0 {
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

fn update_inventory_panel(
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


fn tile_type_to_color(tile: &Tile) -> Color {
    match tile.tile_type {
        '.' => Color::rgb(0.2, 0.8, 0.2), // Green for grass
        'f' => Color::rgb(0.1, 0.5, 0.1), // Dark green for forest
        'M' => Color::rgb(0.5, 0.5, 0.5), // Grey for mountain
        'T' => Color::rgb(0.0, 0.4, 0.0), // Darker green for trees
        '~' => Color::rgb(0.3, 0.3, 0.9), // Blue for water
        '#' => Color::rgb(0.6, 0.4, 0.2), // Brown for road
        'O' => Color::rgb(0.7, 0.7, 0.7), // Light grey for rock
        _ => Color::BLACK,
    }
}
