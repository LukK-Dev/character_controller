use super::{Action, Player};
use crate::physics::CollisionLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use std::{f32::consts::PI, ops::Range};

pub struct PlayerCameraPlugin;

impl Plugin for PlayerCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                orbit,
                follow_player.after(crate::physics::move_and_slide),
                prevent_blindness,
            )
                .chain(),
        );
    }
}

#[derive(Component)]
#[require(Camera3d)]
pub struct PlayerCamera {
    // distance camera tries to stay from player
    target_distance: f32,
    orbit_speed: f32,
    pitch_constraint: Range<f32>,
}

impl Default for PlayerCamera {
    fn default() -> Self {
        Self {
            target_distance: 15.0,
            orbit_speed: PI,
            pitch_constraint: -15f32.to_radians()..90f32.to_radians(),
        }
    }
}

fn orbit(
    mut camera: Query<(&PlayerCamera, &mut Transform)>,
    player: Query<&ActionState<Action>, With<Player>>,
    time: Res<Time>,
) {
    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        if let Ok(input) = player.get_single() {
            let orbit_direction = -input
                .clamped_axis_pair(&Action::CameraOrbit)
                .normalize_or_zero();
            let orbit_angles = orbit_direction * camera.orbit_speed * time.delta_secs();
            let (mut yaw, mut pitch, _) = camera_transform.rotation.to_euler(EulerRot::YXZ);
            pitch += -orbit_angles.y;
            pitch = -(-pitch).clamp(camera.pitch_constraint.start, camera.pitch_constraint.end);
            yaw += orbit_angles.x;
            camera_transform.rotation =
                Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
        }
    }
}

fn follow_player(
    mut camera: Query<(&PlayerCamera, &mut Transform)>,
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
) {
    if let Ok((camera, mut camera_transform)) = camera.get_single_mut() {
        if let Ok(player_transform) = player.get_single() {
            camera_transform.translation =
                player_transform.translation - camera_transform.forward() * camera.target_distance;
        }
    }
}

fn prevent_blindness(
    mut camera: Query<&mut Transform, With<PlayerCamera>>,
    player: Query<(&Transform, &Collider), (With<Player>, Without<PlayerCamera>)>,
    spatial_query: SpatialQuery,
) {
    if let Ok(mut camera_transform) = camera.get_single_mut() {
        if let Ok((player_transform, player_collider)) = player.get_single() {
            let direction = camera_transform.translation - player_transform.translation;
            let distance = direction.length();

            // prevent crash in Dir3::new_unchecked
            if distance == 0.0 {
                return;
            }

            let direction = direction / distance;
            if let Some(hit) = spatial_query.cast_shape(
                // player_collider,
                &Collider::sphere(0.25),
                player_transform.translation,
                player_transform.rotation,
                Dir3::new_unchecked(direction),
                &ShapeCastConfig {
                    max_distance: distance,
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let hit_position = player_transform.translation + direction * hit.distance;
                camera_transform.translation = hit_position;
            }
        }
    }
}
