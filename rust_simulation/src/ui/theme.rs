use bevy::prelude::*;

pub const BACKGROUND_COLOR: Color = Color::rgb(0.1, 0.1, 0.1);

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.35, 0.35);

pub const TEXT_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

pub fn get_title_text_style() -> TextStyle {
    TextStyle {
        font_size: 80.0,
        color: TEXT_COLOR,
        ..default()
    }
}

pub fn get_button_text_style() -> TextStyle {
    TextStyle {
        font_size: 30.0,
        color: TEXT_COLOR,
        ..default()
    }
}
