use crate::state::AppState;
use crate::ui::theme::*;
use bevy::prelude::*;

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), setup_main_menu)
            .add_systems(
                Update,
                (main_menu_button_system, button_hover_system)
                    .run_if(in_state(AppState::MainMenu)),
            );
    }
}

#[derive(Component)]
enum MainMenuButtonAction {
    StartSimulation,
    Settings,
}

fn setup_main_menu(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
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
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Rust Simulation",
                get_title_text_style(),
            ));
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(20.0),
                    ..default()
                },
                ..default()
            });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    MainMenuButtonAction::StartSimulation,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Start Simulation",
                        get_button_text_style(),
                    ));
                });
            parent.spawn(NodeBundle {
                style: Style {
                    height: Val::Px(20.0),
                    ..default()
                },
                ..default()
            });
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(200.0),
                            height: Val::Px(50.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: NORMAL_BUTTON.into(),
                        ..default()
                    },
                    MainMenuButtonAction::Settings,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Settings",
                        get_button_text_style(),
                    ));
                });
        });
}

#[allow(clippy::type_complexity)]
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

#[allow(clippy::type_complexity)]
fn main_menu_button_system(
    interaction_query: Query<
        (&Interaction, &MainMenuButtonAction),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                MainMenuButtonAction::StartSimulation => {
                    app_state.set(AppState::InGame);
                }
                MainMenuButtonAction::Settings => {
                    app_state.set(AppState::Settings);
                }
            }
        }
    }
}
