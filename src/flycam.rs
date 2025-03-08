use std::f32::consts::PI;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};

pub struct FlycamPlugin;

impl Plugin for FlycamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (set_flycam_active, move_flycam));
    }
}

#[derive(Component)]
#[require(Camera3d)]
pub struct Flycam {
    is_active: bool,
    speed: f32,
    speed_multiplier: f32,
    mouse_sensitivity: f32,
}

impl Default for Flycam {
    fn default() -> Self {
        Self {
            is_active: true,
            speed: 10.0,
            speed_multiplier: 2.0,
            mouse_sensitivity: 0.1,
        }
    }
}

fn move_flycam(
    mut flycam: Query<(&Flycam, &mut Transform)>,
    input: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
) {
    if let Ok((flycam, mut transform)) = flycam.get_single_mut() {
        if flycam.is_active {
            let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
            pitch -= (flycam.mouse_sensitivity * mouse_motion.delta.y).to_radians();
            pitch = pitch.clamp(-PI * 0.5, PI * 0.5);
            yaw -= (flycam.mouse_sensitivity * mouse_motion.delta.x).to_radians();
            transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);

            let mut input_direction = Vec3::ZERO;
            if input.pressed(KeyCode::KeyW) {
                input_direction.z -= 1.0;
            }
            if input.pressed(KeyCode::KeyS) {
                input_direction.z += 1.0;
            }
            if input.pressed(KeyCode::KeyA) {
                input_direction.x -= 1.0;
            }
            if input.pressed(KeyCode::KeyD) {
                input_direction.x += 1.0;
            }
            input_direction = input_direction.clamp_length_max(1.0);

            let mut vertical_input = 0.0;
            if input.pressed(KeyCode::Space) {
                vertical_input += 1.0;
            }
            if input.pressed(KeyCode::ControlLeft) {
                vertical_input -= 1.0;
            }

            let speed_multiplier = if input.pressed(KeyCode::ShiftLeft) {
                flycam.speed_multiplier
            } else {
                1.0
            };

            let mut move_direction = transform.rotation * input_direction;
            move_direction.y += vertical_input;
            transform.translation +=
                move_direction * flycam.speed * speed_multiplier * time.delta_secs();
        }
    }
}

fn set_flycam_active(
    mut flycam: Query<&mut Flycam>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut flycam) = flycam.get_single_mut() {
        if input.just_pressed(KeyCode::Escape) {
            flycam.is_active = !flycam.is_active;
        }

        if let Ok(mut window) = window.get_single_mut() {
            window.cursor_options.grab_mode = if flycam.is_active {
                CursorGrabMode::Confined
            } else {
                CursorGrabMode::None
            };

            window.cursor_options.visible = !flycam.is_active;
        }
    }
}
