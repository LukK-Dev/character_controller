use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};

pub struct PhysicsPlugin {
    move_and_slide_iterations: usize,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        let distance = distance_from_center_to_hull(
            &Collider::,
            Quat::IDENTITY,
            Dir3::new_unchecked((Vec3::X + Vec3::Y).normalize()),
        );
        info!("{distance}");

        app.add_plugins(
            PhysicsPlugins::default().set(PhysicsInterpolationPlugin::extrapolate_all()),
            // PhysicsPlugins::default(),
        );
        // app.add_plugins(PhysicsDebugPlugin::default());

        app.insert_resource(MoveAndSlideMaxIterations(self.move_and_slide_iterations));

        // app.add_systems(
        //     PreUpdate,
        //     // (update_grounded_state, apply_gravity, snap_to_floor, unstuck).chain(),
        //     (update_grounded_state, apply_gravity, unstuck).chain(),
        // );
        // app.add_systems(FixedPostUpdate, (move_and_slide, update_grounded_state));

        app.add_systems(Update, move_and_slide_debug_visualization);

        let physics_schedule = app
        .get_schedule_mut(PhysicsSchedule)
            .expect("missing PhysicsSchedule (try adding the Avian PhysicsPlugins before adding this plugin)");
        physics_schedule.add_systems(
            (move_and_slide, respond_to_ground)
                .chain()
                .in_set(PhysicsStepSet::First),
        );
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            move_and_slide_iterations: 8,
        }
    }
}

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Player,
    Terrain,
}

#[derive(Resource)]
pub struct MoveAndSlideMaxIterations(usize);

#[derive(Component)]
#[require(Velocity, Transform, RigidBody(|| RigidBody::Kinematic), Collider)]
pub struct KinematicCharacterBody {
    /// maximum distance between collider and ground for the body to be considered grounded
    grounded_max_distance: f32,
    /// gap between collider of body and terrain to prevent getting stuck
    collider_gap: f32,
    /// angle at which the body will slide off of slopes
    max_terrain_slope: f32,
    snap_to_floor: bool,
    /// maximum distance to floor, at which snapping can occur
    snap_to_floor_max_distance: f32,
}

impl Default for KinematicCharacterBody {
    fn default() -> Self {
        Self {
            grounded_max_distance: 0.1,
            collider_gap: 0.1,
            max_terrain_slope: 45f32.to_radians(),
            snap_to_floor: true,
            snap_to_floor_max_distance: 0.1,
        }
    }
}

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec3);

impl std::ops::Deref for Velocity {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Velocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct Grounded;

pub fn move_and_slide(
    mut bodies: Query<(
        &KinematicCharacterBody,
        &Collider,
        &Velocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideMaxIterations>,
    time: Res<Time>,
) {
    for (body, collider, velocity, mut transform) in bodies.iter_mut() {
        let mut remaining_velocity = velocity.0 * time.delta_secs();
        if remaining_velocity.length_squared() == 0.0 {
            continue;
        }

        let mut cast_position = transform.translation;
        let mut i = 0;
        while i < iterations.0 && remaining_velocity.length_squared() > 0.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                cast_position,
                Quat::IDENTITY,
                Dir3::new_unchecked(remaining_velocity.normalize()),
                &ShapeCastConfig {
                    max_distance: remaining_velocity.length(),
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let movable_distance = hit.distance - body.collider_gap;
                cast_position += remaining_velocity.normalize() * movable_distance;

                let hit_normal_xz_proj = Vec3::new(hit.normal1.x, 0.0, hit.normal1.z).normalize();
                let slope_angle = PI / 2.0 - hit.normal1.angle_between(hit_normal_xz_proj);
                // treat steep slopes and ceilings as walls (planes parallel to y-axis)
                // TODO: investigate if ceilings should be treated as walls (I'm starting to think
                // they shouldn't)
                if slope_angle > body.max_terrain_slope || hit.normal1.y < 0.0 {
                    remaining_velocity =
                        remaining_velocity.reject_from_normalized(hit_normal_xz_proj);
                } else {
                    remaining_velocity = remaining_velocity.reject_from_normalized(hit.normal1);
                }
            } else {
                cast_position += remaining_velocity;
                break;
            }

            i += 1;
        }

        // make sure that collider_gap is kept
        if i > 0 {}

        transform.translation = cast_position;
    }
}


/// add collider_gap to max_distance

pub fn move_and_slide_debug_visualization(
    bodies: Query<(&KinematicCharacterBody, &Collider, &Velocity, &Transform)>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideMaxIterations>,
    mut gizmos: Gizmos,
) {
    for (body, collider, velocity, transform) in bodies.iter() {
        let mut remaining_velocity = velocity.0.normalize() * 10.0;
        if remaining_velocity.length_squared() == 0.0 {
            continue;
        }

        let mut cast_position = transform.translation;
        let mut i = 0;
        while i < iterations.0 && remaining_velocity.length_squared() > 0.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                cast_position,
                Quat::IDENTITY,
                Dir3::new_unchecked(remaining_velocity.normalize()),
                &ShapeCastConfig {
                    max_distance: remaining_velocity.length(),
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let movable_distance = hit.distance - body.collider_gap;
                let last_cast_position = cast_position;
                cast_position += remaining_velocity.normalize() * movable_distance;
                remaining_velocity = remaining_velocity.reject_from_normalized(hit.normal1);

                gizmos.line(last_cast_position, cast_position, tailwind::RED_400);
                gizmos.primitive_3d(
                    &Capsule3d::default(),
                    Isometry3d::from_translation(cast_position),
                    tailwind::GREEN_400,
                );
            } else {
                let last_cast_position = cast_position;
                cast_position += remaining_velocity;

                gizmos.line(last_cast_position, cast_position, tailwind::RED_400);
                gizmos.primitive_3d(
                    &Capsule3d::default(),
                    Isometry3d::from_translation(cast_position),
                    tailwind::GREEN_400,
                );

                break;
            }

            i += 1;
        }
    }
}

fn respond_to_ground(
    mut commands: Commands,
    mut controllers: Query<(
        Entity,
        &KinematicCharacterBody,
        &Collider,
        &Transform,
        &mut Velocity,
    )>,
    spatial_query: SpatialQuery,
) {
    for (entity, body, collider, transform, mut velocity) in controllers.iter_mut() {
        if spatial_query
            .cast_shape(
                collider,
                transform.translation,
                transform.rotation,
                Dir3::new_unchecked(-Vec3::Y),
                &ShapeCastConfig {
                    max_distance: body.grounded_max_distance,
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            )
            .is_some()
        {
            velocity.y = 0.0;
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn distance_from_center_to_hull(collider: &Collider, rotation: Quat, direction: Dir3) -> f32 {
    let aabb = collider.aabb(Vec3::splat(0.0), rotation);
    let max_distance = aabb.min.length().max(aabb.max.length());

    collider
        .cast_ray(
            Vec3::splat(0.0),
            rotation,
            Vec3::splat(0.0),
            direction.into(),
            max_distance,
            false,
        )
        .expect("collider not supported")
        .0
}
