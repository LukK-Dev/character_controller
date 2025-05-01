use crate::physics::KinematicCharacterBody;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Component, InputContext)]
#[require(KinematicCharacterBody, Actions<Player>)]
pub struct Player {
    pub gravity: f32,
    pub acceleration: f32,
    pub max_speed: f32,
    pub sprint_acceleration: f32,
    pub sprint_max_speed: f32,
    pub grounded_deceleration: f32,
    pub jump_impulse: f32,
    pub airborne_acceleration: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            gravity: 9.81,
            acceleration: 30.0,
            max_speed: 7.5,
            sprint_acceleration: 40.0,
            sprint_max_speed: 10.0,
            grounded_deceleration: 30.0,
            jump_impulse: 10.0,
            airborne_acceleration: 15.0,
        }
    }
}

#[derive(Resource)]
pub(super) struct PlayerModel(pub Handle<Scene>);
