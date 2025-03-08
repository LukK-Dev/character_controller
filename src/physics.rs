use avian3d::prelude::*;
use bevy::prelude::*;

#[derive(PhysicsLayer, Default)]
pub enum CollisionLayer {
    #[default]
    Default,
    Player,
    Terrain,
}

#[derive(Component)]
pub struct Grounded;
