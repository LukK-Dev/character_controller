use std::time::Duration;

use avian3d::prelude::*;
use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowMode},
};
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::{
    flycam::FlycamPlugin,
    physics::{CollisionLayer, PhysicsPlugin},
    player::{Player, PlayerPlugin, SpawnPlayer},
};

const PLAYGROUND_SCENE_PATH: &str = "./playground.glb";

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins);
        app.add_plugins(WorldInspectorPlugin::new());
        app.add_plugins(FrameTimeDiagnosticsPlugin);

        app.add_plugins((PlayerPlugin, FlycamPlugin, PhysicsPlugin::default()));

        app.add_systems(Startup, setup);
        app.add_systems(
            Update,
            (
                spawn_spheres,
                fullscreen_on_f11,
                reset_player,
                update_window_title,
            ),
        );
    }
}

fn setup(mut commands: Commands, asset_server: ResMut<AssetServer>) {
    // commands.spawn((
    //     crate::flycam::Flycam::default(),
    //     Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
    // ));

    commands.insert_resource(AmbientLight {
        brightness: 900.0,
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
        ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
            .with_default_layers(CollisionLayers::new(
                CollisionLayer::Terrain,
                LayerMask::ALL,
            )),
        RigidBody::Static,
    ));

    commands.trigger(SpawnPlayer {
        transform: Transform::from_translation(Vec3::new(0.0, 3.0, 0.0)),
    });
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

fn fullscreen_on_f11(
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if input.just_pressed(KeyCode::F11) {
        if let Ok(mut primary_window) = primary_window.get_single_mut() {
            primary_window.mode = match primary_window.mode {
                WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                _ => WindowMode::Windowed,
            }
        }
    }
}

fn reset_player(
    mut player: Query<(&mut Transform, &mut crate::physics::Velocity), With<Player>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if keyboard.just_pressed(KeyCode::KeyR) {
        if let Ok((mut transform, mut velocity)) = player.get_single_mut() {
            transform.translation = Vec3::ZERO.with_y(5.0);
            velocity.0 = Vec3::ZERO;
        }
    }
}

fn _capture_cursor(primary_window: Single<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = primary_window.into_inner();
    primary_window.cursor_options.grab_mode = CursorGrabMode::Locked;
    primary_window.cursor_options.visible = false;
}

fn update_window_title(
    diagnostics: Res<DiagnosticsStore>,
    mut primary_window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if let Some(fps_diagnostic) = diagnostics.get_measurement(&FrameTimeDiagnosticsPlugin::FPS) {
        if let Ok(mut primary_window) = primary_window.get_single_mut() {
            primary_window.title = format!(
                "Character Controller - {} FPS",
                fps_diagnostic.value.round()
            );
        }
    }
}
