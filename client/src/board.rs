use std::f32::consts::PI;

use bevy::prelude::*;
use noughts_and_crosses_core::messages::ClientMessage;

use crate::{client::ClientEvent, game_state::GameState};

pub struct Plugin;

const TILE_SIZE: f32 = 5.0;
const TILE_GAP: f32 = 0.5;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, update)
            .add_event::<HoverEvent>()
            .add_event::<ClickEvent>();
    }
}

#[derive(Event)]
pub struct ClickEvent(pub Vec3);

#[derive(Event)]
pub struct HoverEvent(pub Vec3);

#[derive(Component, Default)]
struct Pos {
    x: usize,
    y: usize,
}

#[derive(Component, Default, Clone)]
enum Tile {
    Cross,
    Nought,
    #[default]
    Board,
}

impl Tile {
    fn transforms(&self, base: &Transform) -> Vec<Transform> {
        match self {
            Self::Cross => {
                let mut left_bar = *base;
                left_bar.scale = Vec3 {
                    x: 1.0,
                    y: 4.0,
                    z: 1.0,
                };
                left_bar.translation.y += 1.0;
                left_bar.rotate_axis(Vec3::Z, PI / 2.0);
                left_bar.rotate_axis(Vec3::Y, PI / 4.0);

                let mut right_bar = left_bar;
                right_bar.rotate_axis(Vec3::Y, PI / 2.0);

                vec![left_bar, right_bar]
            }
            Self::Nought => {
                let mut res = *base;
                res.translation.y += 1.0;
                vec![res]
            }
            Self::Board => vec![*base],
        }
    }
}

#[derive(Bundle, Default)]
struct TileBundle {
    tile: Tile,
    pos: Pos,
    mesh: Handle<Mesh>,
    material: Handle<StandardMaterial>,
    transform: Transform,
    global_transform: GlobalTransform,
    visibility: Visibility,
    inherited_visibility: InheritedVisibility,
    view_visibility: ViewVisibility,
}

#[derive(Resource)]
struct Highlighted {
    material: Handle<StandardMaterial>,
}

#[derive(Resource)]
struct UnHighlighted {
    material: Handle<StandardMaterial>,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let un_highlighted = materials.add(Color::BISQUE.into());
    let highlighted = materials.add(Color::ALICE_BLUE.into());
    let cross_mat = materials.add(Color::RED.into());
    let nought_mat = materials.add(Color::GREEN.into());

    let board_mesh = meshes.add(shape::Plane::from_size(TILE_SIZE).into());
    let cross_mesh = meshes.add(shape::Cylinder::default().into());
    let nought_mesh = meshes.add(shape::Torus::default().into());

    commands.insert_resource(UnHighlighted {
        material: un_highlighted.clone(),
    });
    commands.insert_resource(Highlighted {
        material: highlighted,
    });

    let start_offset = -TILE_SIZE - TILE_GAP;
    for x in 0..3 {
        for y in 0..3 {
            let base = Transform {
                #[allow(clippy::cast_precision_loss)]
                translation: Vec3 {
                    x: (x as f32).mul_add(TILE_GAP + TILE_SIZE, start_offset),
                    y: 0.0,
                    z: (y as f32).mul_add(TILE_GAP + TILE_SIZE, start_offset),
                },
                ..Default::default()
            };

            for tile in [Tile::Board, Tile::Cross, Tile::Nought] {
                for transform in tile.transforms(&base) {
                    commands.spawn(TileBundle {
                        tile: tile.clone(),
                        pos: Pos { x, y },
                        mesh: match tile {
                            Tile::Cross => cross_mesh.clone(),
                            Tile::Nought => nought_mesh.clone(),
                            Tile::Board => board_mesh.clone(),
                        },
                        material: match tile {
                            Tile::Cross => cross_mat.clone(),
                            Tile::Nought => nought_mat.clone(),
                            Tile::Board => un_highlighted.clone(),
                        },
                        transform,
                        visibility: Visibility::Hidden, // the board will be made visible on the first update.
                        ..default()
                    });
                }
            }
        }
    }
}

fn update(
    mut tile_query: Query<(
        &Tile,
        &Pos,
        &GlobalTransform,
        &mut Handle<StandardMaterial>,
        &mut Visibility,
    )>,
    highlighted: Res<Highlighted>,
    un_highlighted: Res<UnHighlighted>,
    mut ev_hover: EventReader<HoverEvent>,
    mut ev_click: EventReader<ClickEvent>,
    mut ev_client: EventWriter<ClientEvent>,
    mut game_state: ResMut<GameState>,
) {
    // the latest hover and click locations if there are any
    let hover = ev_hover.read().last();
    let click = ev_click.read().last();

    let blocked = game_state.blocking_message.is_some();

    for (tile, pos, transform, mut mat, mut visibility) in &mut tile_query {
        let visible = match tile {
            Tile::Cross => game_state.board.tiles[pos.x][pos.y].is_cross(),
            Tile::Nought => game_state.board.tiles[pos.x][pos.y].is_nought(),
            Tile::Board => {
                // the center of the tile
                let centre = transform.translation();

                if let Some(hover) = hover.map(|h| h.0) {
                    if !blocked
                        && game_state.board.tiles[pos.x][pos.y].is_unplayed()
                        && (hover.x - centre.x).abs() < TILE_SIZE / 2.0
                        && (hover.z - centre.z).abs() < TILE_SIZE / 2.0
                    {
                        *mat = highlighted.material.clone();
                    } else {
                        *mat = un_highlighted.material.clone();
                    }
                }

                if !blocked && game_state.board.tiles[pos.x][pos.y].is_unplayed() {
                    if let Some(click) = click.map(|c| c.0) {
                        if (click.x - centre.x).abs() < TILE_SIZE / 2.0
                            && (click.z - centre.z).abs() < TILE_SIZE / 2.0
                        {
                            ev_client.send(
                                ClientMessage::Move((pos.x, pos.y).try_into().unwrap()).into(),
                            );
                        }
                    }
                }

                // logic to request rematch
                if blocked
                    && game_state.board.summary().is_finished()
                    && !game_state.ended
                    && !game_state.rematch_requested
                    && click.is_some()
                {
                    ev_client.send(ClientMessage::RequestRematch.into());
                    game_state.blocking_message =
                        Some("Rematch request sent to opponent".to_string());
                    game_state.rematch_requested = true;
                }

                true
            }
        };

        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}
