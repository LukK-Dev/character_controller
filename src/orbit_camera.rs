use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use std::ops::Range;

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<OrbitCamera>()
            .add_observer(binding)
            .add_systems(Last, follow_target);
    }
}

#[derive(Component, InputContext)]
#[require(Camera3d)]
pub struct OrbitCamera {
    max_distance_to_target: f32,
    pitch_constraint: Range<f32>,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            max_distance_to_target: 10.0,
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

fn follow_target(
    targets: Query<(&Transform, &TargetOf), Changed<Transform>>,
    mut cameras: Query<&mut Transform, (With<OrbitCamera>, Without<Smoothing>)>,
) {
    for (target_transform, target_of) in &targets {
        let mut camera_transform = cameras.get_mut(target_of.0).unwrap();
        *camera_transform = *target_transform;
    }
}
