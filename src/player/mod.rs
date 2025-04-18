mod camera;
pub mod types;

use crate::physics::{CollisionLayer, Grounded, KinematicCharacterBody, Velocity};
use avian3d::prelude::*;
use bevy::prelude::*;
use camera::{PlayerCamera, PlayerCameraPlugin};
use leafwing_input_manager::prelude::*;
use types::{Action, Player, PlayerModel};

// TODO: decouple movement logic from input logic

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<Action>::default());

        app.add_plugins(PlayerCameraPlugin);

        app.add_observer(on_spawn_player);

        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (grounded_movement, airborne_movement, apply_gravity),
        );
    }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_model: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("./models/player/player.glb"));
    commands.insert_resource(PlayerModel(player_model));
}

#[derive(Event)]
pub struct SpawnPlayer {
    pub transform: Transform,
}

fn on_spawn_player(
    trigger: Trigger<SpawnPlayer>,
    player_model: Res<PlayerModel>,
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
        .with_dual_axis(
            Action::CameraOrbit,
            MouseMove::default().inverted_y().sensitivity(0.25),
        );

    let mesh = meshes.add(Capsule3d::new(0.5, 0.9));
    let material = materials.add(StandardMaterial::default());

    commands.spawn((
        Name::new("Player"),
        Player::default(),
        InputManagerBundle::with_map(input_map),
        KinematicCharacterBody::default(),
        Collider::capsule(0.5, 0.9),
        CollisionLayers::new(CollisionLayer::Player, LayerMask::ALL),
        Mesh3d(mesh),
        MeshMaterial3d(material),
        transform,
        PointLight {
            intensity: 100_000.0,
            range: 10.0,
            ..Default::default()
        },
        SceneRoot(player_model.0.clone()),
    ));

    commands.spawn(SceneRoot(player_model.0.clone()));

    // TODO: move to an appropriate spot
    commands.spawn((
        PlayerCamera::default(),
        Transform::from_xyz(0.0, 5.0, 7.5).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        bevy::core_pipeline::smaa::Smaa::default(),
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

fn airborne_movement(
    mut player: Query<
        (&Player, &ActionState<Action>, &mut Transform, &mut Velocity),
        Without<Grounded>,
    >,
    player_camera: Query<&Transform, (With<PlayerCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    if let Ok((player, input, mut transform, mut velocity)) = player.get_single_mut() {
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
            // let target_speed = (velocity.xz().length()
            //     + player.airborne_acceleration * time.delta_secs())
            // .clamp(0.0, player.max_speed);
            // let target_velocity = input_direction * target_speed;
            let target_velocity = (velocity.0.xz()
                + input_direction * player.airborne_acceleration * time.delta_secs())
            .clamp_length_max(player.max_speed);
            velocity.x = target_velocity.x;
            velocity.z = target_velocity.y;
        }
    }
}

fn apply_gravity(mut player: Query<(&Player, &mut Velocity), Without<Grounded>>, time: Res<Time>) {
    if let Ok((player, mut velocity)) = player.get_single_mut() {
        velocity.y -= player.gravity * time.delta_secs();
    }
}
