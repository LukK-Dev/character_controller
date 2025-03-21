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
        app.add_plugins(PhysicsDebugPlugin::default());

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
pub struct MoveAndSlideIterations(usize);

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
            // TODO: RESET TO 0.01
            grounded_max_distance: 0.1,
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

pub fn move_and_slide(
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
    for (controller, collider, velocity, mut transform) in bodies.iter_mut() {
        let mut distance_to_travel = velocity.length() * time.delta_secs();
        if distance_to_travel.is_nan() || distance_to_travel == 0.0 {
            continue;
        }

        let mut cast_position = transform.translation;
        let mut cast_direction = velocity.normalize();
        let mut i = 0;
        while i < iterations.0 && distance_to_travel > 0.0 {
            let last_cast_position = cast_position;
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
                let movable_distance =
                    (hit.distance - controller.collider_gap).min(distance_to_travel);
                distance_to_travel -= movable_distance;
                cast_position += cast_direction * movable_distance;
                gizmos.primitive_3d(
                    &Capsule3d::new(0.5, 1.0),
                    Isometry3d::new(cast_position, transform.rotation),
                    tailwind::GREEN_400,
                );
                gizmos.arrow(last_cast_position, cast_position, tailwind::RED_400);

                // project direction vector onto plane defined by hit normal
                let new_cast_direction =
                    (cast_direction - hit.normal1 * hit.normal1.dot(cast_direction)).normalize();
                if !new_cast_direction.is_nan() {
                    cast_direction = new_cast_direction;
                }
            } else {
                cast_position += cast_direction * distance_to_travel;
                distance_to_travel = 0.0;
                gizmos.primitive_3d(
                    &Capsule3d::new(0.5, 1.0),
                    Isometry3d::new(cast_position, transform.rotation),
                    tailwind::GREEN_400,
                );
                gizmos.arrow(last_cast_position, cast_position, tailwind::RED_400);
            }

            i += 1;
        }

        transform.translation = cast_position;
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
