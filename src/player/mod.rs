use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::{flycam::Flycam, physics::CollisionLayer};

const MOVE_AND_SLIDE_MAX_ITERATIONS: usize = 8;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());

        app.add_observer(on_spawn_player);
        app.add_systems(Update, movement);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    #[actionlike(DualAxis)]
    Move,
    Jump,
}

#[derive(Component)]
pub struct Player {
    acceleration: f32,
    grounded_deceleration: f32,
    gravity: f32,
    max_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            acceleration: 20.0,
            grounded_deceleration: 20.0,
            gravity: 9.81,
            max_speed: 5.0,
        }
    }
}

#[derive(Event)]
pub struct SpawnPlayer {
    pub transform: Transform,
}

fn on_spawn_player(
    trigger: Trigger<SpawnPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let transform = trigger.event().transform;

    let input_map = InputMap::default()
        .with_dual_axis(
            Action::Move,
            GamepadStick::LEFT.with_deadzone_symmetric(0.1),
        )
        .with_dual_axis(Action::Move, VirtualDPad::wasd())
        .with(Action::Jump, GamepadButton::East)
        .with(Action::Jump, KeyCode::Space);
    let mesh = meshes.add(Capsule3d::new(0.5, 1.0));
    let material = materials.add(StandardMaterial {
        base_color: tailwind::RED_400.into(),
        ..Default::default()
    });

    commands.spawn((
        Player::default(),
        Name::new("Player"),
        InputManagerBundle::with_map(input_map),
        transform,
        RigidBody::Kinematic,
        Collider::capsule(0.5, 1.0),
        CollisionLayers::new(CollisionLayer::Player, CollisionLayer::Terrain),
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
    ));
}

fn movement(
    mut player: Query<(
        &Player,
        &ActionState<Action>,
        &mut Transform,
        &mut LinearVelocity,
    )>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    if let Ok((player, input, mut transform, mut velocity)) = player.get_single_mut() {
        gizmos.arrow(
            transform.translation,
            transform.translation + transform.forward().as_vec3() * 2.0,
            tailwind::BLUE_400,
        );

        let mut input_direction = input.clamped_axis_pair(&Action::Move).normalize_or_zero();
        input_direction.y = -input_direction.y;
        if input_direction.length_squared() > 0.0 {
            // rotation
            transform.look_to(
                Dir3::new_unchecked(Vec3::new(input_direction.x, 0.0, input_direction.y)),
                Dir3::new_unchecked(Vec3::Y),
            );

            // movement
            // let acceleration = (transform.rotation
            //     * Vec3::new(input_direction.x, 0.0, input_direction.y)
            //     * player.acceleration
            //     * time.delta_secs())
            // .xz();
            let acceleration = (Vec3::new(input_direction.x, 0.0, input_direction.y)
                * player.acceleration
                * time.delta_secs())
            .xz();
            let target_velocity = (velocity.xz() + acceleration).clamp_length_max(player.max_speed);
            velocity.x = target_velocity.x;
            velocity.z = target_velocity.y;
        } else {
            // apply ground friction
            let decelerated_speed =
                velocity.xz().length() - player.grounded_deceleration * time.delta_secs();
            let mut decelerated_velocity = Vec2::ZERO;
            if decelerated_speed > 0.0 {
                decelerated_velocity = velocity.xz().clamp_length_max(decelerated_speed);
            }
            velocity.x = decelerated_velocity.x;
            velocity.z = decelerated_velocity.y;
        }
    }
}
