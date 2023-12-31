use bevy::{input::touch::TouchPhase, prelude::*, window::PrimaryWindow};

use crate::board::{ClickEvent, HoverEvent};

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(
    camera_query: Query<(&Camera, &mut GlobalTransform)>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    buttons: Res<Input<MouseButton>>,
    mut ev_click: EventWriter<ClickEvent>,
    mut ev_hover: EventWriter<HoverEvent>,
    mut touch_evr: EventReader<TouchInput>,
) {
    let (camera, camera_global_transform) = camera_query.single();
    let window = window_query.single();

    let mut click_position = None;

    // read touches
    for ev in touch_evr.read() {
        if ev.phase == TouchPhase::Started {
            click_position = Some(ev.position);
        }
    }

    // read mouse clicks
    if buttons.just_pressed(MouseButton::Left) {
        if let Some(pos) = window.cursor_position() {
            click_position = Some(pos);
        }
    }

    // process hover
    if let Some(pos) = window.cursor_position() {
        if let Some(ray) = camera.viewport_to_world(camera_global_transform, pos) {
            if let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) {
                let point = ray.get_point(distance);
                ev_hover.send(HoverEvent(point));
            }
        }
    }

    // process the latest touch or mouse click
    if let Some(pos) = click_position {
        if let Some(ray) = camera.viewport_to_world(camera_global_transform, pos) {
            if let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) {
                let point = ray.get_point(distance);
                ev_click.send(ClickEvent(point));
            }
        }
    }
}
