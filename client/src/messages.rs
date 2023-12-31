use bevy::prelude::*;

use crate::game_state::GameState;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup)
            .add_systems(Update, update);
    }
}

#[derive(Component)]
enum UiText {
    Blocking,
    Turn,
}

fn startup(mut commands: Commands) {
    // blocking text
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 32.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            ..default()
        }),
        UiText::Blocking,
    ));

    // turn text
    commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 24.0,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(5.0),
            left: Val::Px(5.0),
            ..default()
        }),
        UiText::Turn,
    ));
}

fn update(game_state: Res<GameState>, mut query: Query<(&mut Text, &UiText)>) {
    for (mut text, ui_text) in &mut query {
        match ui_text {
            UiText::Blocking => {
                text.sections[0].value = game_state.blocking_message.clone().unwrap_or_default();
            }
            UiText::Turn => {
                text.sections[0].value = if game_state.blocking_message.is_some() {
                    ""
                } else if game_state.is_crosses {
                    "Your turn. You are crosses."
                } else {
                    "Your turn. You are noughts."
                }
                .to_string();
            }
        }
    }
}
