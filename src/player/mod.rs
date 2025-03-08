use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};
use leafwing_input_manager::prelude::*;

use crate::physics::{self, CollisionLayer, Grounded};

const MOVE_AND_SLIDE_MAX_ITERATIONS: usize = 8;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());

        app.add_observer(on_spawn_player);
        app.add_systems(Update, movement);
        // app.add_systems(FixedPostUpdate, (movement, move_and_slide));
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
    max_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            acceleration: 1.0,
            max_speed: 3.0,
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
    let mut input_map = InputMap::default()
        .with_dual_axis(Action::Move, GamepadStick::LEFT)
        .with_dual_axis(Action::Move, VirtualDPad::wasd());
    input_map.insert(Action::Jump, GamepadButton::East);
    input_map.insert(Action::Jump, KeyCode::Space);

    let transform = trigger.event().transform;
    let mesh = meshes.add(Capsule3d::new(0.5, 1.0));
    let material = materials.add(StandardMaterial {
        base_color: tailwind::RED_400.into(),
        ..Default::default()
    });
    commands.spawn((
        Player::default(),
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
        &mut Transform,
        &mut LinearVelocity,
        &ActionState<Action>,
    )>,
    time: Res<Time>,
) {
    if let Ok((player, mut transform, mut velocity, input)) = player.get_single_mut() {
        let input_direction = input.clamped_axis_pair(&Action::Move).normalize();
        // if input_direction.length_squared() == 0.0 {
        //     velocity.0.x = 0.0;
        //     velocity.0.z = 0.0;
        //     return;
        // }

        let move_direction =
            transform.rotation * Vec3::new(input_direction.x, 0.0, input_direction.y);
        let acceleration = move_direction * player.acceleration * time.delta_secs();
        velocity.x += acceleration.x;
        velocity.z += acceleration.z;
        // let _ = velocity.0.clamp_length_max(player.max_speed);

        // velocity.0 += move_direction * player.speed * time.delta_secs();
        // transform.translation += velocity.0;
    }
}

fn apply_gravity(mut player: Query<(&Player, &mut LinearVelocity)>, time: Res<Time>) {}
