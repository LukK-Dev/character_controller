use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};
use std::f32::consts::{FRAC_PI_2, PI};
// TODO: apply small offset to avoid extended collider from penetrating surfaces

// small number used to work around floating point inaccuracies
const EPSILON: f32 = 1e-04;

pub struct PhysicsPlugin {
    collide_and_slide_max_iterations: usize,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // let collider = Collider::capsule(1.0, 1.0);
        // println!(
        //     "{:?}",
        //     distance_from_center_to_hull(&collider, Quat::IDENTITY, Dir3::new_unchecked(Vec3::Y))
        // );
        // let collider = inflated_collider(&collider, -0.5);
        // println!(
        //     "{:?}",
        //     distance_from_center_to_hull(&collider, Quat::IDENTITY, Dir3::new_unchecked(Vec3::Y))
        // );

        app.add_plugins((
            // PhysicsPlugins::default().set(PhysicsInterpolationPlugin::extrapolate_all()),
            PhysicsPlugins::default(),
            // PhysicsDebugPlugin::default(),
        ));

        app.insert_resource(CollideAndSlideMaxIterations(
            self.collide_and_slide_max_iterations,
        ));

        app.add_systems(
            PostUpdate,
            (
                collide_and_slide,
                snap_to_floor,
                collide_and_slide_debug_visualization,
                respond_to_ground,
            )
                .chain(),
        );

        // app.add_systems(
        //     PhysicsSchedule,
        //     (collide_and_slide, respond_to_ground)
        //         .chain()
        //         .in_set(PhysicsStepSet::First),
        // );
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            collide_and_slide_max_iterations: 8,
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
pub struct CollideAndSlideMaxIterations(usize);

#[derive(Component)]
#[require(Velocity, Transform, RigidBody(|| RigidBody::Kinematic), Collider)]
pub struct KinematicCharacterBody {
    /// maximum distance between collider and ground for the body to be considered grounded
    grounded_max_distance: f32,
    /// gap between collider of body and terrain to prevent getting stuck
    // collider_gap: f32,
    /// body will slide off of terrain with slope angles greater than ['max_terrain_slope']
    max_terrain_slope: f32,
    snap_to_floor: bool,
    /// maximum distance to floor, at which snapping can occur
    snap_to_floor_max_distance: f32,
}

impl Default for KinematicCharacterBody {
    fn default() -> Self {
        Self {
            grounded_max_distance: 0.05,
            // collider_gap: 0.015,
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

pub fn collide_and_slide(
    mut bodies: Query<(
        &KinematicCharacterBody,
        &Collider,
        &Velocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
    max_iterations: Res<CollideAndSlideMaxIterations>,
    time: Res<Time>,
) {
    bodies
        .par_iter_mut()
        .for_each(|(_body, collider, velocity, mut transform)| {
            let mut remaining_velocity = velocity.0 * time.delta_secs();
            let mut remaining_distance = remaining_velocity.length();
            let adjusted_collider = inflated_collider(collider, -EPSILON);
            let mut position = transform.translation;
            let mut direction = remaining_velocity.normalize();
            let mut i = 0;
            while i < max_iterations.0 && remaining_distance > 0.0 {
                if let Some(hit) = spatial_query.cast_shape(
                    &adjusted_collider,
                    position,
                    Quat::IDENTITY,
                    Dir3::new_unchecked(direction),
                    &ShapeCastConfig {
                        max_distance: remaining_distance + EPSILON,
                        ..Default::default()
                    },
                    &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
                ) {
                    let mut new_position = position + direction * hit.distance;
                    new_position += hit.normal1 * EPSILON;

                    remaining_distance -= position.distance(new_position);
                    remaining_velocity =
                        remaining_velocity.normalize_or_zero() * remaining_distance;
                    remaining_velocity = remaining_velocity.reject_from_normalized(hit.normal1);
                    direction = remaining_velocity.normalize();

                    position = new_position;
                } else {
                    position += remaining_velocity;
                    break;
                }

                i += 1;
            }

            transform.translation = position;
        });
}

pub fn collide_and_slide_debug_visualization(
    bodies: Query<(&KinematicCharacterBody, &Collider, &Velocity, &Transform)>,
    spatial_query: SpatialQuery,
    max_iterations: Res<CollideAndSlideMaxIterations>,
    mut gizmos: Gizmos,
) {
    for (_body, collider, velocity, transform) in &bodies {
        let mut remaining_velocity = velocity.0;
        let adjusted_collider = inflated_collider(collider, -EPSILON);
        let mut position = transform.translation;

        let capsule = adjusted_collider.shape().as_capsule().unwrap();
        gizmos.primitive_3d(
            &Capsule3d::new(capsule.radius, capsule.height()),
            Isometry3d::from_translation(position),
            tailwind::GREEN_500,
        );

        let mut direction = remaining_velocity.normalize();
        let mut i = 0;
        while i < max_iterations.0 && remaining_velocity.length_squared() > 0.0 {
            if let Some(hit) = spatial_query.cast_shape(
                &adjusted_collider,
                position,
                Quat::IDENTITY,
                Dir3::new_unchecked(direction),
                &ShapeCastConfig {
                    max_distance: remaining_velocity.length() + EPSILON,
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let mut new_position = position + direction * hit.distance;
                new_position += hit.normal1 * EPSILON;

                gizmos.arrow(position, new_position, tailwind::RED_500);
                gizmos.primitive_3d(
                    &Capsule3d::new(capsule.radius, capsule.height()),
                    Isometry3d::from_translation(new_position),
                    tailwind::GREEN_500,
                );

                remaining_velocity = remaining_velocity.reject_from_normalized(hit.normal1);
                direction = remaining_velocity.normalize();

                position = new_position;
            } else {
                let new_position = position + remaining_velocity;
                gizmos.arrow(position, new_position, tailwind::RED_500);
                gizmos.primitive_3d(
                    &Capsule3d::new(capsule.radius, capsule.height()),
                    Isometry3d::from_translation(new_position),
                    tailwind::GREEN_500,
                );
                break;
            }

            i += 1;
        }
    }
}

pub fn snap_to_floor(
    mut bodies: Query<(
        &KinematicCharacterBody,
        &Collider,
        &Velocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
) {
    // bodies
    //     .par_iter_mut()
    //     .for_each(|(_body, collider, velocity, mut transform)| {}
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
        let adjusted_collider = inflated_collider(collider, -EPSILON);
        let mut is_grounded = false;
        if let Some(hit) = spatial_query.cast_shape(
            &adjusted_collider,
            transform.translation,
            transform.rotation,
            Dir3::new_unchecked(-Vec3::Y),
            &ShapeCastConfig {
                max_distance: body.grounded_max_distance + EPSILON,
                ..Default::default()
            },
            &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
        ) {
            let ground_angle = hit.normal1.angle_between(Vec3::Y);
            is_grounded = ground_angle <= body.max_terrain_slope;
        }

        if is_grounded {
            velocity.y = 0.0;
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}

fn distance_from_center_to_hull(
    collider: &Collider,
    collider_rotation: Quat,
    direction: Dir3,
) -> f32 {
    let aabb = collider.aabb(Vec3::splat(0.0), collider_rotation);
    let max_distance = aabb.min.length().max(aabb.max.length());

    collider
        .cast_ray(
            Vec3::splat(0.0),
            collider_rotation,
            Vec3::splat(0.0),
            direction.into(),
            max_distance,
            false,
        )
        .expect("collider not supported")
        .0
}

fn inflated_collider(collider: &Collider, size: f32) -> Collider {
    if let Some(ball) = collider.shape().as_ball() {
        Collider::sphere(ball.radius + size)
    } else if let Some(capsule) = collider.shape().as_capsule() {
        let extended_radius = capsule.radius + size;
        let inclusive_height = capsule.height() + 2.0 * capsule.radius;
        let inclusive_extended_height = inclusive_height + 2.0 * size;
        let extended_height = inclusive_extended_height - 2.0 * extended_radius;
        Collider::capsule(extended_radius, extended_height)
    } else {
        panic!("unsupported shape");
    }
}
