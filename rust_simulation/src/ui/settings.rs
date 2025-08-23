use crate::config::Config;
use crate::state::AppState;
use bevy::input::prelude::ButtonInput;
use bevy::prelude::*;
use bevy::reflect::{Reflect, ReflectRef};

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Settings), setup_settings_menu)
            .add_systems(
                Update,
                (
                    settings_button_system,
                    text_input_system,
                    save_button_system,
                )
                    .run_if(in_state(AppState::Settings)),
            )
            .add_systems(OnExit(AppState::Settings), cleanup_settings_menu);
    }
}

#[derive(Component)]
struct SettingsUi;

#[derive(Component)]
enum SettingsButtonAction {
    Back,
    Save,
}

#[derive(Component)]
struct ConfigField {
    path: String,
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

            // Dynamically create the UI
            let config_reflect = config.as_reflect();
            build_ui_for_struct(parent, config_reflect, "", 0);

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

fn build_ui_for_struct(
    parent: &mut ChildBuilder,
    value: &dyn Reflect,
    path: &str,
    depth: usize,
) {
    if let ReflectRef::Struct(s) = value.reflect_ref() {
        for (i, field) in s.iter_fields().enumerate() {
            let field_name = s.name_at(i).unwrap();
            let new_path = if path.is_empty() {
                field_name.to_string()
            } else {
                format!("{}.{}", path, field_name)
            };

            parent
                .spawn(NodeBundle {
                    style: Style {
                        margin: UiRect::new(Val::Px(20.0 * depth as f32), Val::Px(0.0), Val::Px(10.0), Val::Px(0.0)),
                        flex_direction: FlexDirection::Row,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        format!("{}:", field_name),
                        TextStyle {
                            font_size: 20.0,
                            ..default()
                        },
                    ));

                    if let ReflectRef::Struct(_) = field.reflect_ref() {
                        build_ui_for_struct(parent, field, &new_path, depth + 1);
                    } else {
                        let field_value = format!("{:?}", field.as_any());
                        parent.spawn((
                            TextBundle::from_section(
                                field_value,
                                TextStyle {
                                    font_size: 20.0,
                                    ..default()
                                },
                            ),
                            ConfigField { path: new_path },
                        ));
                    }
                });
        }
    }
}

fn cleanup_settings_menu(mut commands: Commands, query: Query<Entity, With<SettingsUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn text_input_system(
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_text: Query<(&mut Text, &ConfigField)>,
    mut config: ResMut<Config>,
) {
    for (mut text, config_field) in q_text.iter_mut() {
        if keys.just_pressed(KeyCode::Backspace) {
            text.sections[0].value.pop();
        }
        for ev in char_evr.read() {
            text.sections[0].value.push_str(&ev.char);
        }

        let field_path = &config_field.path;
        let mut config_reflect = config.as_mut();
        if let Some(field) = config_reflect.field_mut(field_path) {
            if field.is::<u32>() {
                if let Ok(value) = text.sections[0].value.parse::<u32>() {
                    *field.downcast_mut::<u32>().unwrap() = value;
                }
            } else if field.is::<f64>() {
                if let Ok(value) = text.sections[0].value.parse::<f64>() {
                    *field.downcast_mut::<f64>().unwrap() = value;
                }
            } else if field.is::<String>() {
                *field.downcast_mut::<String>().unwrap() = text.sections[0].value.clone();
            }
        }
    }
}

fn save_button_system(
    interaction_query: Query<(&Interaction, &SettingsButtonAction), (Changed<Interaction>, With<Button>)>,
    config: Res<Config>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let SettingsButtonAction::Save = action {
                let data = toml::to_string(&*config).unwrap();
                std::fs::write("data/config.toml", data).unwrap();
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
