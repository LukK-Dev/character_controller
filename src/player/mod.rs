mod camera;

use crate::physics::{CollisionLayer, Grounded, KinematicCharacterBody, Velocity};
use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};
use camera::{PlayerCamera, PlayerCameraPlugin};
use leafwing_input_manager::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());

        app.add_plugins(PlayerCameraPlugin);

        app.add_observer(on_spawn_player);

        app.add_systems(Update, (grounded_movement, apply_gravity));
        // app.add_systems(Update, apply_gravity);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
enum Action {
    #[actionlike(DualAxis)]
    Move,
    Jump,
    Sprint,

    #[actionlike(DualAxis)]
    CameraOrbit,
}

#[derive(Component)]
#[require(KinematicCharacterBody)]
pub struct Player {
    gravity: f32,
    acceleration: f32,
    max_speed: f32,
    sprint_acceleration: f32,
    sprint_max_speed: f32,
    grounded_deceleration: f32,
    jump_impulse: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            acceleration: 20.0,
            max_speed: 5.0,
            sprint_acceleration: 30.0,
            sprint_max_speed: 7.5,
            grounded_deceleration: 30.0,
            jump_impulse: 10.0,
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
            GamepadStick::LEFT.with_deadzone_symmetric(0.2),
        )
        .with_dual_axis(Action::Move, VirtualDPad::wasd())
        .with(Action::Jump, GamepadButton::East)
        .with(Action::Jump, KeyCode::Space)
        .with(Action::Sprint, GamepadButton::South)
        .with(Action::Sprint, KeyCode::ShiftLeft)
        .with_dual_axis(
            Action::CameraOrbit,
            GamepadStick::RIGHT.with_deadzone_symmetric(0.2),
        )
        .with_dual_axis(Action::CameraOrbit, MouseMove::default().inverted_y());

    let mesh = meshes.add(Capsule3d::new(0.5, 1.0));
    let material = materials.add(StandardMaterial {
        base_color: tailwind::RED_400.into(),
        ..Default::default()
    });

    commands.spawn((
        Name::new("Player"),
        Player::default(),
        InputManagerBundle::with_map(input_map),
        KinematicCharacterBody::default(),
        Collider::capsule(0.5, 1.0),
        CollisionLayers::new(CollisionLayer::Player, LayerMask::ALL),
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        transform,
    ));

    // TODO: move to an appropriate spot
    commands.spawn((
        PlayerCamera::default(),
        bevy::core_pipeline::smaa::Smaa::default(),
        Transform::from_xyz(0.0, 7.5, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));
}

fn grounded_movement(
    mut player: Query<
        (&Player, &ActionState<Action>, &mut Transform, &mut Velocity),
        With<Grounded>,
    >,
    player_camera: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok((player, input, mut transform, mut velocity)) = player.get_single_mut() {
        if input.just_pressed(&Action::Jump) {
            velocity.y = player.jump_impulse;
        }

        let mut input_direction = input.clamped_axis_pair(&Action::Move).normalize_or_zero();
        input_direction.y = -input_direction.y;
        if input_direction.length_squared() > 0.0 {
            // rotation
            transform.look_to(
                Dir3::new_unchecked(Vec3::new(input_direction.x, 0.0, input_direction.y)),
                Dir3::new_unchecked(Vec3::Y),
            );

            // adjust rotation to take player camera rotation into account
            if let Ok(player_camera_transform) = player_camera.get_single() {
                let (yaw, _, _) = player_camera_transform.rotation.to_euler(EulerRot::YXZ);
                let camera_rotation_offset = Quat::from_axis_angle(Vec3::Y, yaw);
                transform.rotate(camera_rotation_offset);
                input_direction = Mat2::from_angle(-yaw) * input_direction;
            }

            // basic horizontal movement
            let mut acceleration = player.acceleration;
            let mut max_speed = player.max_speed;
            if input.pressed(&Action::Sprint) {
                acceleration = player.sprint_acceleration;
                max_speed = player.sprint_max_speed;
            }

            let target_speed =
                (velocity.xz().length() + acceleration * time.delta_secs()).clamp(0.0, max_speed);
            let target_velocity = input_direction * target_speed;
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

fn apply_gravity(mut player: Query<(&Player, &mut Velocity), Without<Grounded>>, time: Res<Time>) {
    if let Ok((player, mut velocity)) = player.get_single_mut() {
        velocity.y -= player.gravity * time.delta_secs();
    }
}
