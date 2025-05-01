mod camera;
mod input;
pub mod types;

use crate::{
    orbit_camera::{OrbitCamera, TargetOf},
    physics::{CollisionLayer, Grounded, KinematicCharacterBody, Velocity},
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;
use input::*;
use std::f32::consts::PI;
use types::{Player, PlayerModel};

// TODO: decouple movement logic from input logic

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_input_context::<Player>()
            .add_observer(binding)
            .add_observer(on_spawn_player)
            .add_observer(jump)
            .add_systems(Startup, setup)
            .add_systems(
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
    // TODO: move to an appropriate spot
    let camera = commands
        .spawn((
            OrbitCamera::default(),
            bevy::core_pipeline::smaa::Smaa::default(),
        ))
        .id();

    let transform = trigger.event().transform;

    let mesh = meshes.add(Capsule3d::new(0.3, 1.3));
    let material = StandardMaterial {
        base_color: Color::WHITE.with_alpha(0.5),
        alpha_mode: AlphaMode::Blend,
        ..Default::default()
    };
    let material = materials.add(material);

    let player = commands
        .spawn((
            Name::new("Player"),
            Player::default(),
            Actions::<Player>::default(),
            KinematicCharacterBody::default(),
            Collider::capsule(0.3, 1.3),
            CollisionLayers::new(CollisionLayer::Player, LayerMask::ALL),
            Mesh3d(mesh),
            MeshMaterial3d(material),
            transform,
            PointLight {
                intensity: 10_000.0,
                range: 5.0,
                ..Default::default()
            },
            TargetOf(camera),
        ))
        .id();
    commands.spawn((
        SceneRoot(player_model.0.clone()),
        Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, PI))
            .with_translation(Vec3::NEG_Y * (0.3 + 0.65)),
        ChildOf(player),
    ));
}

fn grounded_movement(
    player: Single<
        (
            &Player,
            &Actions<Player>,
            &TargetOf,
            &mut Transform,
            &mut Velocity,
        ),
        With<Grounded>,
    >,
    cameras: Query<&Transform, (With<OrbitCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let (player, actions, target_of, mut transform, mut velocity) = player.into_inner();
    let mut input_direction = actions
        .action::<input::Move>()
        .value()
        .as_axis2d()
        .normalize_or_zero();
    input_direction.y = -input_direction.y;
    if input_direction.length_squared() > 0.0 {
        // rotation
        transform.look_to(
            Dir3::new_unchecked(Vec3::new(input_direction.x, 0.0, input_direction.y)),
            Dir3::new_unchecked(Vec3::Y),
        );

        // adjust rotation to take player camera rotation into account
        if let Ok(player_camera_transform) = cameras.get(target_of.0) {
            let (yaw, _, _) = player_camera_transform.rotation.to_euler(EulerRot::YXZ);
            let camera_rotation_offset = Quat::from_axis_angle(Vec3::Y, yaw);
            transform.rotate(camera_rotation_offset);
            input_direction = Mat2::from_angle(-yaw) * input_direction;
        }

        // basic horizontal movement
        let mut acceleration = player.acceleration;
        let mut max_speed = player.max_speed;
        if actions.action::<Sprint>().state() == ActionState::Fired {
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

fn airborne_movement(
    player: Single<
        (
            &Player,
            &Actions<Player>,
            &TargetOf,
            &mut Transform,
            &mut Velocity,
        ),
        Without<Grounded>,
    >,
    cameras: Query<&Transform, (With<OrbitCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let (player, actions, target_of, mut transform, mut velocity) = player.into_inner();
    let mut input_direction = actions
        .action::<input::Move>()
        .value()
        .as_axis2d()
        .normalize_or_zero();
    input_direction.y = -input_direction.y;
    if input_direction.length_squared() > 0.0 {
        // rotation
        transform.look_to(
            Dir3::new_unchecked(Vec3::new(input_direction.x, 0.0, input_direction.y)),
            Dir3::new_unchecked(Vec3::Y),
        );

        // adjust rotation to take player camera rotation into account
        if let Ok(player_camera_transform) = cameras.get(target_of.0) {
            let (yaw, _, _) = player_camera_transform.rotation.to_euler(EulerRot::YXZ);
            let camera_rotation_offset = Quat::from_axis_angle(Vec3::Y, yaw);
            transform.rotate(camera_rotation_offset);
            input_direction = Mat2::from_angle(-yaw) * input_direction;
        }

        // basic horizontal movement
        let target_velocity = (velocity.0.xz()
            + input_direction * player.airborne_acceleration * time.delta_secs())
        .clamp_length_max(player.max_speed);
        velocity.x = target_velocity.x;
        velocity.z = target_velocity.y;
    }
}

fn jump(
    trigger: Trigger<Fired<Jump>>,
    mut players: Query<(&Player, &mut Velocity), With<Grounded>>,
) {
    if let Ok((player, mut velocity)) = players.get_mut(trigger.target()) {
        velocity.y += player.jump_impulse;
    }
}

fn apply_gravity(player: Single<(&Player, &mut Velocity), Without<Grounded>>, time: Res<Time>) {
    let (player, mut velocity) = player.into_inner();
    velocity.y -= player.gravity * time.delta_secs();
}
