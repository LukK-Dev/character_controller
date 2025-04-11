use crate::physics::KinematicCharacterBody;
use bevy::prelude::*;
use leafwing_input_manager::prelude::*;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub(super) enum Action {
    #[actionlike(DualAxis)]
    Move,
    Jump,
    Sprint,

    #[actionlike(DualAxis)]
    CameraOrbit,
}

#[derive(Component)]
#[require(KinematicCharacterBody)]
pub struct Player {
    pub gravity: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub sprint_acceleration: f32,
    pub sprint_max_speed: f32,
    pub grounded_deceleration: f32,
    pub jump_impulse: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            acceleration: 20.0,
            max_speed: 5.0,
            sprint_acceleration: 30.0,
            sprint_max_speed: 7.5,
            grounded_deceleration: 30.0,
            jump_impulse: 10.0,
        }
    }
}

#[derive(Resource)]
pub(super) struct PlayerModel(pub Handle<Scene>);
