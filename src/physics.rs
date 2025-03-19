use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};

pub struct PhysicsPlugin {
    move_and_slide_iterations: usize,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MoveAndSlideIterations(self.move_and_slide_iterations));

        app.add_systems(PreUpdate, (update_grounded_state, apply_gravity).chain());
        app.add_systems(PostUpdate, move_and_slide);

        // let physics_schedule = app
        // .get_schedule_mut(PhysicsSchedule)
        //     .expect("missing PhysicsSchedule (try adding the Avian PhysicsPlugins before adding this plugin)");
        // physics_schedule.add_systems(move_and_slide.before(PhysicsStepSet::First));
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
struct MoveAndSlideIterations(usize);

#[derive(Component)]
#[require(DesiredVelocity, Transform, RigidBody(|| RigidBody::Kinematic), Collider)]
pub struct KinematicCharacterController {
    gravity: f32,
    // maximum distance between collider and ground for the body to be considered grounded
    grounded_max_distance: f32,
    // gap between collider of character and environment to prevent getting stuck
    collider_gap: f32,
}

impl Default for KinematicCharacterController {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            grounded_max_distance: 0.01,
            collider_gap: 0.01,
        }
    }
}

#[derive(Default, Component)]
pub struct DesiredVelocity(Vec3);

impl std::ops::Deref for DesiredVelocity {
    type Target = Vec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for DesiredVelocity {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Component)]
pub struct Grounded;

fn move_and_slide(
    mut bodies: Query<(
        &KinematicCharacterController,
        &Collider,
        &DesiredVelocity,
        &mut Transform,
    )>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideIterations>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (controller, collider, desired_velocity, mut transform) in bodies.iter_mut() {
        if desired_velocity.is_nan() || desired_velocity.length_squared() == 0.0 {
            return;
        }

        // let mut distance_to_move = desired_velocity.length() * time.delta_secs();
        let mut distance_to_move = desired_velocity.length();
        let mut direction = desired_velocity.normalize();
        let mut slide_position = transform.translation;
        let mut i = 0;
        while
        /* distance_to_move > 0.0 && */
        i < iterations.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                slide_position,
                // TODO: should ideally be changed every iteration
                // my character uses a capsule collider, for whom it wouldn't change anything
                transform.rotation,
                Dir3::new_unchecked(direction),
                &ShapeCastConfig {
                    max_distance: distance_to_move,
                    ..Default::default()
                },
                // TODO: could be made generic (I won't)
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let movable_distance =
                    hit.distance.clamp(0.0, distance_to_move) - controller.collider_gap;
                distance_to_move -= movable_distance;

                let _slide_pos = slide_position;
                slide_position += direction * movable_distance;

                gizmos.line(_slide_pos, slide_position, tailwind::GREEN_400);

                gizmos.sphere(
                    Isometry3d::from_translation(slide_position),
                    0.5,
                    tailwind::GREEN_400,
                );

                // project direction vector onto plane defined by hit normal
                // TODO: find out why this does not work as i want it
                direction =
                    (direction + hit.normal1 * hit.normal1.dot(direction).abs()).normalize();
                if direction.is_nan() {
                    break;
                }
            } else {
                slide_position = transform.translation + direction * distance_to_move;
                distance_to_move = 0.0;
            }

            i += 1;
        }

        transform.translation += desired_velocity.0 * time.delta_secs();

        // transform.translation = slide_position;

        // let slide_velocity =
        //     (slide_position - transform.translation).normalize() * velocity.normalize();
        // velocity.0 = slide_velocity;
    }
}

fn apply_gravity(
    mut controllers: Query<
        (&KinematicCharacterController, &mut DesiredVelocity),
        Without<Grounded>,
    >,
    time: Res<Time>,
) {
    for (controller, mut velocity) in controllers.iter_mut() {
        velocity.y -= controller.gravity * time.delta_secs();
    }
}

fn update_grounded_state(
    mut commands: Commands,
    controllers: Query<(Entity, &KinematicCharacterController, &Transform, &Collider)>,
    spatial_query: SpatialQuery,
) {
    for (entity, controller, transform, collider) in controllers.iter() {
        if spatial_query
            .cast_shape(
                collider,
                transform.translation,
                transform.rotation,
                Dir3::new_unchecked(-Vec3::Y),
                &ShapeCastConfig {
                    max_distance: controller.grounded_max_distance,
                    ..Default::default()
                },
                &SpatialQueryFilter {
                    mask: LayerMask(CollisionLayer::Terrain.to_bits()),
                    ..Default::default()
                },
            )
            .is_some()
        {
            commands.entity(entity).insert(Grounded);
        } else {
            commands.entity(entity).remove::<Grounded>();
        }
    }
}
