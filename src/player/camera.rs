use super::{Action, Player};
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use std::f32::consts::PI;

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (follow_player.after(crate::physics::move_and_slide), orbit).chain(),
        );
    }
}

#[derive(Component)]
#[require(Camera3d)]
pub struct PlayerCamera {
    // distance camera tries to stay from player
    target_distance: f32,
    orbit_speed: f32,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            target_distance: 5.0,
            orbit_speed: PI / 2.0,
        }
    }
}

fn follow_player(
    mut camera: Query<(&PlayerCamera, &mut Transform)>,
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    if let Ok((_camera, mut camera_transform)) = camera.get_single_mut() {
        if let Ok(player_transform) = player.get_single() {
            camera_transform.translation = player_transform.translation;
        }
    }
}

fn orbit(
    mut camera: Query<(&PlayerCamera, &mut Transform)>,
    player: Query<(&Transform, &ActionState<Action>), (With<Player>, Without<PlayerCamera>)>,
    time: Res<Time>,
) {
    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        if let Ok((player_transform, input)) = player.get_single() {
            let orbit_direction = input
                .clamped_axis_pair(&Action::CameraOrbit)
                .normalize_or_zero();
            camera_transform.rotate_y(-orbit_direction.x * camera.orbit_speed * time.delta_secs());
            camera_transform
                .rotate_local_x(orbit_direction.y * camera.orbit_speed * time.delta_secs());
        }
    }
}
