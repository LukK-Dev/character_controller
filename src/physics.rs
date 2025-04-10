use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};

pub struct PhysicsPlugin {
    move_and_slide_iterations: usize,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            // PhysicsPlugins::default().set(PhysicsInterpolationPlugin::extrapolate_all()),
            PhysicsPlugins::default(),
        );
        // app.add_plugins(PhysicsDebugPlugin::default());

        app.insert_resource(MoveAndSlideIterations(self.move_and_slide_iterations));

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
pub struct MoveAndSlideIterations(usize);

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
    iterations: Res<MoveAndSlideIterations>,
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
                remaining_velocity = remaining_velocity.reject_from_normalized(hit.normal1);
            } else {
                cast_position += remaining_velocity;
                break;
            }

            i += 1;
        }

        transform.translation = cast_position;
    }
}

pub fn move_and_slide_debug_visualization(
    mut bodies: Query<(&KinematicCharacterBody, &Collider, &Velocity, &Transform)>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideIterations>,
    mut gizmos: Gizmos,
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut distance_to_move: Local<Option<f32>>,
) {
    return;
    let distance_to_move = distance_to_move.get_or_insert(5.0);
    for event in mouse_wheel_events.read() {
        *distance_to_move += event.y * 0.25;
    }

    for (body, collider, velocity, transform) in bodies.iter_mut() {
        if velocity.length_squared() == 0.0 {
            continue;
        }

        let mut distance_to_move = *distance_to_move;
        let mut cast_direction = velocity.normalize();
        let mut cast_position = transform.translation;
        let mut i = 0;
        while i < iterations.0 && distance_to_move > 0.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                cast_position,
                Quat::IDENTITY,
                Dir3::new_unchecked(cast_direction),
                &ShapeCastConfig {
                    max_distance: distance_to_move,
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let movable_distance = (hit.distance - body.collider_gap).min(distance_to_move);
                distance_to_move -= movable_distance;
                let new_cast_position = cast_position + cast_direction * movable_distance;
                cast_direction = cast_direction
                    .reject_from_normalized(hit.normal1)
                    .normalize();
                gizmos.line(cast_position, new_cast_position, tailwind::RED_400);
                gizmos.primitive_3d(
                    &Capsule3d::default(),
                    Isometry3d::from_translation(new_cast_position),
                    tailwind::GREEN_400,
                );
                cast_position = new_cast_position;
            } else {
                let new_cast_position = transform.translation + cast_direction * distance_to_move;
                distance_to_move = 0.0;
                gizmos.line(cast_position, new_cast_position, tailwind::RED_400);
                gizmos.primitive_3d(
                    &Capsule3d::default(),
                    Isometry3d::from_translation(new_cast_position),
                    tailwind::GREEN_400,
                );
            }

            i += 1;
        }
    }
}

pub fn _move_and_slide(
    mut bodies: Query<(
        &KinematicCharacterBody,
        &Collider,
        &Velocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideIterations>,
    time: Res<Time>,
) {
    for (body, collider, velocity, mut transform) in bodies.iter_mut() {
        let mut distance_to_travel = velocity.length() * time.delta_secs();
        if distance_to_travel.is_nan() || distance_to_travel == 0.0 {
            continue;
        }

        let mut cast_position = transform.translation;
        let mut cast_direction = velocity.normalize();
        let mut i = 0;
        while i < iterations.0 && distance_to_travel > 0.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                cast_position,
                // TODO: should ideally be changed every iteration
                // my character uses a capsule collider, for whom rotating around the y-axis wouldn't change anything
                transform.rotation,
                Dir3::new_unchecked(cast_direction),
                &ShapeCastConfig {
                    max_distance: distance_to_travel,
                    ..Default::default()
                },
                // TODO: could be made generic (I won't)
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let movable_distance = (hit.distance - body.collider_gap).min(distance_to_travel);
                distance_to_travel -= movable_distance;
                cast_position += cast_direction * movable_distance;

                // project direction vector onto plane defined by hit normal
                let mut new_cast_direction = cast_direction
                    .reject_from_normalized(hit.normal1)
                    .normalize();
                if !new_cast_direction.is_nan() {
                    // treat sloped ceilings as walls
                    if new_cast_direction.y < 0.0 {
                        new_cast_direction.y = 0.0;
                        new_cast_direction = new_cast_direction.normalize();
                    }

                    cast_direction = new_cast_direction;
                }
            } else {
                cast_position += cast_direction * distance_to_travel;
                distance_to_travel = 0.0;
            }

            i += 1;
        }

        transform.translation = cast_position;
    }
}

fn respond_to_ground(
    mut commands: Commands,
    mut controllers: Query<(
        Entity,
        &KinematicCharacterBody,
        &Collider,
        &mut Velocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
) {
    for (entity, body, collider, mut velocity, mut transform) in controllers.iter_mut() {
        if let Some(hit) = spatial_query.cast_shape(
            collider,
            transform.translation,
            transform.rotation,
            Dir3::new_unchecked(-Vec3::Y),
            &ShapeCastConfig {
                max_distance: body.grounded_max_distance,
                ..Default::default()
            },
            &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
        ) {
            velocity.y = 0.0;
            // TODO:
            // keep collider_gap
            // transform.translation.y += controller.collider_gap - hit.distance;
            // transform.translation.y = hit.point2.y + body.collider_gap + 1.0;
            info!("distance to ground: {}", transform.translation.y - 1.0);
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
            info!("airborne");
        }
    }
}
