use std::time::Duration;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    flycam::{Flycam, FlycamPlugin},
    player::{Player, PlayerPlugin, SpawnPlayer},
};

const PLAYGROUND_SCENE_PATH: &str = "./playground.glb";

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(PhysicsDebugPlugin::default());
        app.add_plugins(WorldInspectorPlugin::new());

        app.add_plugins((PlayerPlugin, FlycamPlugin));

        app.add_systems(Startup, setup);
        app.add_systems(Update, (move_and_slide, spawn_spheres));
    }
}

// fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Flycam::default(),
        // Transform::from_xyz(0.0, 10.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        brightness: 400.0,
        ..Default::default()
    });
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, 0.72, 3.11, 0.0)),
    ));

    commands.spawn((
        SceneRoot(asset_server.load(GltfAssetLabel::Scene(0).from_asset(PLAYGROUND_SCENE_PATH))),
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh),
        RigidBody::Static,
    ));

    commands.trigger(SpawnPlayer {
        transform: Transform::from_translation(Vec3::new(0.0, 1.0, 0.0)),
    });

    // MOVE AND SLIDE TEST
    // commands.spawn((
    //     Transform::from_rotation(Quat::from_axis_angle(Vec3::Y, -45.0f32.to_radians())),
    //     Mesh3d(meshes.add(Rectangle::new(3.0, 3.0))),
    //     MeshMaterial3d::<StandardMaterial>(materials.add(StandardMaterial::default())),
    //     RigidBody::Static,
    //     Collider::cuboid(3.0, 3.0, 0.01),
    // ));
}

fn move_and_slide(// mut gizmos: Gizmos,
    // spatial_query: SpatialQuery,
    // mut walls: Query<&mut Transform, (With<Collider>, Without<Player>)>,
) {
    // for mut transform in walls.iter_mut() {
    //     transform.rotate_y(0.5 / 75.0);
    // }
    //
    // let collider_radius = 0.5;
    // let position = Vec3::new(0.0, 0.0, 2.5);
    // let direction = Vec3::new(0.0, 0.0, -1.0);
    // let time_step = 1.0;
    // let speed = 5.0;
    // let velocity = direction * speed * time_step;
    // let mut distance_to_travel = velocity.length();
    //
    // gizmos.sphere(
    //     Isometry3d::from_translation(position),
    //     collider_radius,
    //     Color::WHITE,
    // );
    // gizmos.sphere(
    //     Isometry3d::from_translation(
    //         player::physics::move_and_slide(
    //             Transform::from_translation(position),
    //             velocity,
    //             &Collider::sphere(collider_radius),
    //             spatial_query,
    //         )
    //         .translation,
    //     ),
    //     collider_radius,
    //     Color::WHITE,
    // );

    // gizmos.arrow(position, position + velocity, tailwind::YELLOW_400);
    //
    // if let Some(hit) = spatial_query.cast_shape(
    //     &Collider::sphere(collider_radius),
    //     position,
    //     Quat::IDENTITY,
    //     Dir3::new(direction).unwrap(),
    //     &ShapeCastConfig {
    //         max_distance: distance_to_travel,
    //         ..Default::default()
    //     },
    //     &SpatialQueryFilter::default(),
    // ) {
    //     let hit_position = position + direction * hit.distance;
    //     gizmos.sphere(
    //         Isometry3d::from_translation(hit_position),
    //         collider_radius,
    //         tailwind::RED_400,
    //     );
    //
    //     let slide_direction =
    //         (direction + hit.normal1 * hit.normal1.dot(direction).abs()).normalize();
    //     distance_to_travel -= hit.distance;
    //     let slide_position = hit_position + slide_direction * distance_to_travel;
    //     gizmos.sphere(
    //         Isometry3d::from_translation(slide_position),
    //         collider_radius,
    //         tailwind::RED_400,
    //     );
    //     gizmos.arrow(hit_position, slide_position, tailwind::YELLOW_400);
    // }
}

fn spawn_spheres(
    mut commands: Commands,
    input: Res<ButtonInput<MouseButton>>,
    camera: Query<&Transform, With<Camera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut timer: Local<Option<Timer>>,
) {
    let timer = timer.get_or_insert(Timer::new(Duration::from_secs_f32(0.1), TimerMode::Once));
    timer.tick(time.delta());
    let material = StandardMaterial::default();
    let material = materials.add(material);
    if input.pressed(MouseButton::Left) && timer.finished() {
        let sphere_mesh = meshes.add(Sphere::new(0.5));
        let camera_transform = camera.get_single().unwrap();
        commands.spawn((
            Mesh3d(sphere_mesh),
            MeshMaterial3d(material),
            RigidBody::Dynamic,
            Collider::sphere(0.5),
            *camera_transform,
            ExternalImpulse::new(camera_transform.forward().as_vec3() * 10.0),
        ));
        timer.reset();
    }
}
