use crate::config::Config;
use crate::state::AppState;
use crate::ui::theme::*;
use bevy::input::prelude::ButtonInput;
use bevy::prelude::*;
use bevy::reflect::ReflectRef;

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
                    button_hover_system,
                    focus_system,
                    checkbox_system,
                    validation_feedback_system,
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
    Revert,
}

#[derive(Component)]
struct ConfigField {
    path: String,
}

#[derive(Component, Default)]
struct Focused;

#[derive(Component, PartialEq)]
enum ValidationState {
    Valid,
    Invalid,
}

#[derive(Resource)]
struct OriginalConfig(Config);

fn setup_settings_menu(mut commands: Commands, config: Res<Config>) {
    commands.insert_resource(OriginalConfig(config.clone()));

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
                background_color: BACKGROUND_COLOR.into(),
                ..default()
            },
            SettingsUi,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Settings",
                get_title_text_style(),
            ));

            // Dynamically create the UI
            let config_reflect = config.as_reflect();
            build_ui_for_struct(parent, config_reflect, "", 0);

            parent
                .spawn((
                    ButtonBundle {
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    SettingsButtonAction::Save,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Save",
                        get_button_text_style(),
                    ));
                });
            parent
                .spawn((
                    ButtonBundle {
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    SettingsButtonAction::Revert,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Revert",
                        get_button_text_style(),
                    ));
                });
            parent
                .spawn((
                    ButtonBundle {
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    SettingsButtonAction::Back,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Back",
                        get_button_text_style(),
                    ));
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
                            color: TEXT_COLOR,
                            ..default()
                        },
                    ));

                    if let ReflectRef::Struct(_) = field.reflect_ref() {
                        build_ui_for_struct(parent, field, &new_path, depth + 1);
                    } else if field.is::<bool>() {
                        let value = *field.downcast_ref::<bool>().unwrap();
                        parent.spawn((
                            ButtonBundle {
                                style: Style { ..default() },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            TextBundle::from_section(
                                value.to_string(),
                                TextStyle {
                                    font_size: 20.0,
                                    color: TEXT_COLOR,
                                    ..default()
                                },
                            ),
                            ConfigField { path: new_path },
                            ValidationState::Valid,
                        ));
                    } else {
                        let field_value = format!("{:?}", field.as_any());
                        parent.spawn((
                            ButtonBundle {
                                style: Style { ..default() },
                                background_color: NORMAL_BUTTON.into(),
                                ..default()
                            },
                            TextBundle::from_section(
                                field_value,
                                TextStyle {
                                    font_size: 20.0,
                                    color: TEXT_COLOR,
                                    ..default()
                                },
                            ),
                            ConfigField { path: new_path },
                            ValidationState::Valid,
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

fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        *color = match *interaction {
            Interaction::Pressed => PRESSED_BUTTON.into(),
            Interaction::Hovered => HOVERED_BUTTON.into(),
            Interaction::None => NORMAL_BUTTON.into(),
        }
    }
}

fn focus_system(
    mut commands: Commands,
    interaction_query: Query<(Entity, &Interaction), (Changed<Interaction>, With<ConfigField>)>,
    mut focused_query: Query<(Entity, &mut BackgroundColor), With<Focused>>,
) {
    for (entity, interaction) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            // Remove focus from any other field
            for (focused_entity, mut bg_color) in focused_query.iter_mut() {
                *bg_color = NORMAL_BUTTON.into();
                commands.entity(focused_entity).remove::<Focused>();
            }
            // Add focus to the clicked field
            commands.entity(entity).insert(Focused);
        }
    }
}

fn text_input_system(
    mut char_evr: EventReader<ReceivedCharacter>,
    keys: Res<ButtonInput<KeyCode>>,
    mut q_text: Query<(&mut Text, &ConfigField, &mut ValidationState), With<Focused>>, // Only query for focused fields
    mut config: ResMut<Config>,
) {
    for (mut text, config_field, mut validation_state) in q_text.iter_mut() {
        if keys.just_pressed(KeyCode::Backspace) {
            text.sections[0].value.pop();
        }
        for ev in char_evr.read() {
            text.sections[0].value.push_str(&ev.char);
        }

        let field_path = &config_field.path;
        let config_reflect = config.as_mut();
        if let Some(field) = config_reflect.field_mut(field_path) {
            if field.is::<u32>() {
                if let Ok(value) = text.sections[0].value.parse::<u32>() {
                    *field.downcast_mut::<u32>().unwrap() = value;
                    *validation_state = ValidationState::Valid;
                } else {
                    *validation_state = ValidationState::Invalid;
                }
            } else if field.is::<f64>() {
                if let Ok(value) = text.sections[0].value.parse::<f64>() {
                    *field.downcast_mut::<f64>().unwrap() = value;
                    *validation_state = ValidationState::Valid;
                } else {
                    *validation_state = ValidationState::Invalid;
                }
            } else if field.is::<String>() {
                *field.downcast_mut::<String>().unwrap() = text.sections[0].value.clone();
                *validation_state = ValidationState::Valid;
            }
        }
    }
}

fn validation_feedback_system(mut q_text: Query<(&mut Text, &ValidationState)>) {
    for (mut text, validation_state) in q_text.iter_mut() {
        if *validation_state == ValidationState::Valid {
            text.sections[0].style.color = TEXT_COLOR;
        } else {
            text.sections[0].style.color = Color::RED;
        }
    }
}

fn checkbox_system(
    interaction_query: Query<(Entity, &Interaction, &ConfigField), (Changed<Interaction>, With<Button>)>,
    mut config: ResMut<Config>,
    mut text_query: Query<&mut Text>,
    children_query: Query<&Children>,
) {
    for (entity, interaction, config_field) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let config_reflect = config.as_mut();
            if let Some(field) = config_reflect.field_mut(&config_field.path) {
                if field.is::<bool>() {
                    let value = field.downcast_mut::<bool>().unwrap();
                    *value = !*value;

                    // Update the text of the button
                    if let Ok(children) = children_query.get(entity) {
                        for &child in children.iter() {
                            if let Ok(mut text) = text_query.get_mut(child) {
                                text.sections[0].value = value.to_string();
                            }
                        }
                    }
                }
            }
        }
    }
}

fn save_button_system(
    interaction_query: Query<(&Interaction, &SettingsButtonAction), (Changed<Interaction>, With<Button>)>,
    config: Res<Config>,
    q_validation: Query<&ValidationState>,
    mut q_button: Query<(&mut Visibility, &SettingsButtonAction)>,
) {
    let mut all_valid = true;
    for validation_state in q_validation.iter() {
        if *validation_state == ValidationState::Invalid {
            all_valid = false;
            break;
        }
    }

    for (mut visibility, action) in q_button.iter_mut() {
        if let SettingsButtonAction::Save = action {
            if all_valid {
                *visibility = Visibility::Visible;
            } else {
                *visibility = Visibility::Hidden;
            }
        }
    }

    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            if let SettingsButtonAction::Save = action {
                if all_valid {
                    let data = toml::to_string(&*config).unwrap();
                    std::fs::write("data/config.toml", data).unwrap();
                }
            }
        }
    }
}

fn settings_button_system(
    interaction_query: Query<(&Interaction, &SettingsButtonAction), (Changed<Interaction>, With<Button>)>,
    mut app_state: ResMut<NextState<AppState>>,
    mut config: ResMut<Config>,
    original_config: Res<OriginalConfig>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                SettingsButtonAction::Back => {
                    app_state.set(AppState::MainMenu);
                }
                SettingsButtonAction::Revert => {
                    *config = original_config.0.clone();
                }
                _ => {}
            }
        }
    }
}
