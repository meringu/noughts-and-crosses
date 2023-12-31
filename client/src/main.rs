#![warn(clippy::all, clippy::nursery, clippy::pedantic)]
#![allow(clippy::needless_pass_by_value)] // bevy does this for immutable resources and queries

mod board;
mod camera;
mod client;
mod game_state;
mod input;
mod messages;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(board::Plugin)
        .add_plugins(camera::Plugin)
        .add_plugins(client::Plugin)
        .add_plugins(game_state::Plugin)
        .add_plugins(input::Plugin)
        .add_plugins(messages::Plugin)
        .run();
}
