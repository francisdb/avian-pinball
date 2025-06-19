use crate::{PLAYFIELD_CENTER, PlayerCamera};
use bevy::app::{App, Plugin, Update};
use bevy::input::ButtonInput;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::math::{Dir3, Quat, Vec2, Vec3};
use bevy::prelude::{EventReader, KeyCode, MouseButton, Query, Res, Transform, With};

pub(crate) struct ControlCameraPlugin;

impl Plugin for ControlCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, camera_scroll_up_down)
            .add_systems(Update, camera_mouse_rotate);
    }
}

fn camera_scroll_up_down(
    mut scroll_evr: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let mut scroll_delta = 0.0;
    for ev in scroll_evr.read() {
        scroll_delta += ev.y;
    }
    if scroll_delta != 0.0 {
        for mut transform in &mut query {
            // Move camera up/down
            transform.translation.y += scroll_delta * 0.05;
            // Re-apply looking at the playfield center
            *transform = Transform::from_translation(transform.translation)
                .looking_at(PLAYFIELD_CENTER, Dir3::Y);
        }
    }
}

fn camera_mouse_rotate(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<PlayerCamera>>,
) {
    let mut move_delta = Vec3::ZERO;
    // Camera translation with Ctrl + mouse drag
    if mouse_button_input.pressed(MouseButton::Left) && keyboard.pressed(KeyCode::ControlLeft) {
        for ev in mouse_motion_events.read() {
            // Move along X and Y axes (screen space)
            move_delta.x -= ev.delta.x * 0.01; // sensitivity
            move_delta.y += ev.delta.y * 0.01;
        }
        if move_delta != Vec3::ZERO {
            for mut transform in &mut query {
                transform.translation += move_delta;
            }
        }
    } else if mouse_button_input.pressed(MouseButton::Left) {
        // Camera rotation (yaw/pitch) with left mouse drag
        let mut delta = Vec2::ZERO;
        for ev in mouse_motion_events.read() {
            delta += ev.delta;
        }
        if delta != Vec2::ZERO {
            for mut transform in &mut query {
                // Yaw (around up axis)
                let yaw = -delta.x * 0.01; // sensitivity
                // Pitch (around right axis)
                let pitch = -delta.y * 0.01;
                let right = transform.rotation * Vec3::X;
                let up = Vec3::Y;
                let translation = transform.translation;
                // Apply yaw
                transform.rotate_around(translation, Quat::from_axis_angle(up, yaw));
                // Apply pitch
                transform.rotate_around(translation, Quat::from_axis_angle(right, pitch));
            }
        }
    }
}
