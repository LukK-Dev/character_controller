use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};

pub struct PhysicsPlugin {
    move_and_slide_iterations: usize,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MoveAndSlideIterations(self.move_and_slide_iterations));

        app.add_systems(PreUpdate, (update_grounded_state, apply_gravity).chain());

        let physics_schedule = app
            .get_schedule_mut(PhysicsSchedule)
            .expect("missing PhysicsSchedule (try adding the Avian PhysicsPlugins before adding this plugin)");
        physics_schedule.add_systems(move_and_slide.before(PhysicsStepSet::First));
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

#[derive(Component)]
#[require(Transform, RigidBody(|| RigidBody::Kinematic), Collider)]
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

#[derive(Resource)]
struct MoveAndSlideIterations(usize);

#[derive(Component)]
pub struct Grounded;

fn move_and_slide(
    mut bodies: Query<(
        &KinematicCharacterController,
        &Collider,
        &mut Transform,
        &mut LinearVelocity,
    )>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideIterations>,
    time: Res<Time>,
) {
    for (controller, collider, mut transform, mut velocity) in bodies.iter_mut() {
        if velocity.is_nan() || velocity.length_squared() == 0.0 {
            return;
        }

        let mut distance_to_move = velocity.length() * time.delta_secs();
        let mut direction = velocity.normalize();
        let mut slide_position = transform.translation;
        let mut i = 0;
        while distance_to_move <= 0.0 && i < iterations.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                slide_position,
                transform.rotation,
                Dir3::new_unchecked(direction),
                &ShapeCastConfig {
                    max_distance: distance_to_move,
                    ..Default::default()
                },
                &SpatialQueryFilter::from_mask(CollisionLayer::Terrain),
            ) {
                let movable_distance = hit
                    .distance
                    .clamp(0.0, velocity.length() - controller.collider_gap);
                distance_to_move -= movable_distance;
                // project direction vector onto plane defined by hit normal
                direction =
                    (direction + hit.normal1 * hit.normal1.dot(direction).abs()).normalize();
                slide_position = transform.translation + direction * hit.distance;
                i += 1;
            }
        }

        let slide_velocity =
            (slide_position - transform.translation).normalize() * velocity.normalize();
        // velocity.0 = slide_velocity;
    }
}

fn apply_gravity(
    mut controllers: Query<(&KinematicCharacterController, &mut LinearVelocity), Without<Grounded>>,
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
