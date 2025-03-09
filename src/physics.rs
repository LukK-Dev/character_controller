use avian3d::prelude::*;
use bevy::{color::palettes::tailwind, prelude::*};

pub struct PhysicsPlugin {
    move_and_slide_iterations: usize,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MoveAndSlideIterations(self.move_and_slide_iterations));

        app.add_systems(PreUpdate, update_grounded_state);

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
#[require(Transform, Collider, LinearVelocity, GroundedMaxDistance)]
pub struct MoveAndSlide;

#[derive(Resource)]
struct MoveAndSlideIterations(usize);

#[derive(Component)]
pub struct Grounded;

// maximum distance between collider and ground for the body to be considered grounded
#[derive(Component)]
pub struct GroundedMaxDistance(f32);

impl Default for GroundedMaxDistance {
    fn default() -> Self {
        Self(0.01)
    }
}

fn move_and_slide(
    mut bodies: Query<(&Transform, &Collider, &mut LinearVelocity), With<MoveAndSlide>>,
    spatial_query: SpatialQuery,
    iterations: Res<MoveAndSlideIterations>,
    time: Res<Time>,
    mut gizmos: Gizmos,
) {
    for (transform, collider, mut velocity) in bodies.iter_mut() {
        if velocity.length_squared() == 0.0 || velocity.is_nan() {
            return;
        }

        let mut distance_to_travel = velocity.length() * time.delta_secs();
        let mut direction = velocity.normalize();
        info!("{:?}", direction);
        let mut hit_position = transform.translation;
        let mut i = 0;
        while distance_to_travel > 0.0 && i < iterations.0 {
            if let Some(hit) = spatial_query.cast_shape(
                collider,
                hit_position,
                // should be updated every iteration (I won't)
                transform.rotation,
                Dir3::new_unchecked(direction),
                &ShapeCastConfig {
                    max_distance: distance_to_travel,
                    ..Default::default()
                },
                &SpatialQueryFilter {
                    mask: LayerMask(CollisionLayer::Terrain.to_bits()),
                    ..Default::default()
                },
            ) {
                hit_position += direction * hit.distance;
                gizmos.sphere(
                    Isometry3d {
                        translation: hit_position.into(),
                        ..Default::default()
                    },
                    0.5,
                    tailwind::GREEN_400,
                );
                // project movement vector onto plane defined by hit normal
                direction =
                    (direction + hit.normal1 * hit.normal1.dot(direction).abs()).normalize();
                distance_to_travel -= hit.distance;
            } else {
                hit_position += direction * distance_to_travel
            }

            i += 1;
        }

        let mut slid_velocity =
            (hit_position - transform.translation).normalize_or_zero() * velocity.length();
        velocity.0 = slid_velocity;
    }
}

fn update_grounded_state(
    mut commands: Commands,
    bodies: Query<(Entity, &Transform, &Collider, &GroundedMaxDistance), With<MoveAndSlide>>,
    spatial_query: SpatialQuery,
) {
    for (entity, transform, collider, max_distance) in bodies.iter() {
        if spatial_query
            .cast_shape(
                collider,
                transform.translation,
                transform.rotation,
                Dir3::new_unchecked(-Vec3::Y),
                &ShapeCastConfig {
                    max_distance: max_distance.0,
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
