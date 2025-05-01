use crate::physics::CollisionLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use std::{f32::consts::PI, ops::Range};

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<OrbitCamera>()
            .add_observer(binding)
            .add_systems(Last, (orbit, follow_target, prevent_blindness).chain());
    }
}

#[derive(Component, InputContext)]
#[require(Camera3d, Actions<OrbitCamera>)]
pub struct OrbitCamera {
    orbit_radius: f32,
    orbit_speed: f32,
    pitch_constraint: Range<f32>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            orbit_radius: 10.0,
            orbit_speed: PI,
            pitch_constraint: -15f32.to_radians()..90f32.to_radians(),
        }
    }
}

/// If this component is added to an [`OrbitCamera`], the movement to keep the target in the center
/// of the camera will be smoothed.
#[derive(Component)]
pub struct Smoothing {
    decay_rate: f32,
}

impl Default for Smoothing {
    fn default() -> Self {
        Self {
            decay_rate: f32::ln(10.0),
        }
    }
}

#[derive(Component)]
pub struct PreventBlindness {
    camera_collider: Collider,
}

impl Default for PreventBlindness {
    fn default() -> Self {
        Self {
            camera_collider: Collider::sphere(0.25),
        }
    }
}

#[derive(Component)]
#[relationship(relationship_target = Target)]
pub struct TargetOf(pub Entity);

#[derive(Component)]
#[relationship_target(relationship = TargetOf)]
pub struct Target(Entity);

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
struct Orbit;

fn binding(trigger: Trigger<Binding<OrbitCamera>>, mut cameras: Query<&mut Actions<OrbitCamera>>) {
    let mut actions = cameras.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Orbit>()
        .to((Input::mouse_motion(), Axial::right_stick()))
        .with_modifiers(DeadZone::default());
}

fn orbit(
    mut cameras: Query<(&mut Transform, &OrbitCamera, &Actions<OrbitCamera>)>,
    time: Res<Time>,
) {
    for (mut transform, camera, actions) in &mut cameras {
        let orbit_direction = -actions
            .action::<Orbit>()
            .value()
            .as_axis2d()
            .normalize_or_zero();
        info!(?orbit_direction);
        let orbit_angles = orbit_direction * camera.orbit_speed * time.delta_secs();
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
        pitch += -orbit_angles.y;
        pitch = -(-pitch).clamp(camera.pitch_constraint.start, camera.pitch_constraint.end);
        yaw += orbit_angles.x;
        transform.rotation =
            Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
    }
}

fn follow_target(
    mut cameras: Query<(&mut Transform, &OrbitCamera)>,
    targets: Query<(&Transform, &TargetOf), Without<OrbitCamera>>,
) {
    for (target_transform, target_of) in &targets {
        let (mut camera_transform, camera) = cameras.get_mut(target_of.0).unwrap();
        camera_transform.translation = target_transform.translation
            - camera_transform.forward().as_vec3() * camera.orbit_radius;
    }
}

fn prevent_blindness(
    mut cameras: Query<(&mut Transform, &PreventBlindness), With<OrbitCamera>>,
    targets: Query<(&Transform, &TargetOf), Without<OrbitCamera>>,
    spatial_query: SpatialQuery,
) {
    for (target_transform, target_of) in &targets {
        let (mut camera_transform, pb) = cameras.get_mut(target_of.0).unwrap();
        let direction = camera_transform.translation - target_transform.translation;
        let distance = direction.length();

        // prevent crash in Dir3::new_unchecked
        if distance == 0.0 {
            return;
        }

        let direction = direction / distance;
        if let Some(hit) = spatial_query.cast_shape(
            &pb.camera_collider,
            target_transform.translation,
            target_transform.rotation,
            Dir3::new_unchecked(direction),
            &ShapeCastConfig {
                max_distance: distance,
                ..Default::default()
            },
            &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
        ) {
            let hit_position = target_transform.translation + direction * hit.distance;
            camera_transform.translation = hit_position;
        }
    }
}
