use crate::config::Config;
use crate::state::AppState;
use bevy::input::prelude::ButtonInput;
use bevy::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Settings), setup_settings_menu)
            .add_systems(
                Update,
                (
                    text_input_system,
                    save_button_system,
                    settings_button_system,
                )
                    .run_if(in_state(AppState::Settings)),
            )
            .add_systems(OnExit(AppState::Settings), cleanup_settings_menu);
    }
}

#[derive(Component)]
struct SettingsUi;

#[derive(Component)]
struct NumPlayersInput;

#[derive(Component)]
enum SettingsButtonAction {
    Back,
    Save,
}

fn setup_settings_menu(mut commands: Commands, config: Res<Config>) {
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                ..default()
            },
            SettingsUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Settings",
                TextStyle {
                    font_size: 80.0,
                    ..default()
                },
            ));
            // Num Players
            parent
                .spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Number of Players:",
                        TextStyle {
                            font_size: 20.0,
                            ..default()
                        },
                    ));
                    parent.spawn((
                        TextBundle::from_section(
                            config.player_settings.num_players.to_string(),
                            TextStyle {
                                font_size: 20.0,
                                ..default()
                            },
                        ),
                        NumPlayersInput,
                    ));
                });

            parent
                .spawn((
                    ButtonBundle { ..default() },
                    SettingsButtonAction::Save,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Save", TextStyle { ..default() }));
                });
            parent
                .spawn((
                    ButtonBundle { ..default() },
                    SettingsButtonAction::Back,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section("Back", TextStyle { ..default() }));
                });
        });
}

fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn text_input_system(
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_text: Query<&mut Text, With<NumPlayersInput>>,
) {
    if let Some(mut text) = q_text.iter_mut().next() {
        if keys.just_pressed(KeyCode::Backspace) {
            text.sections[0].value.pop();
        }
        for ev in char_evr.read() {
            if ev.char.chars().all(char::is_numeric) {
                text.sections[0].value.push_str(&ev.char);
            }
        }
    }
}

fn save_button_system(
    interaction_query: Query<(&Interaction, &SettingsButtonAction), (Changed<Interaction>, With<Button>)>,
    q_text: Query<&Text, With<NumPlayersInput>>,
    mut config: ResMut<Config>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let SettingsButtonAction::Save = action {
                if let Some(text) = q_text.iter().next() {
                    if let Ok(num_players) = text.sections[0].value.parse() {
                        config.player_settings.num_players = num_players;
                        // Save to file
                        let data = toml::to_string(&*config).unwrap();
                        std::fs::write("data/config.toml", data).unwrap();
                    }
                }
            }
        }
    }
}

fn settings_button_system(
    interaction_query: Query<(&Interaction, &SettingsButtonAction), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let SettingsButtonAction::Back = action {
                app_state.set(AppState::MainMenu);
            }
        }
    }
}
