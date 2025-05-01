#![allow(dead_code, clippy::type_complexity)]

mod flycam;
mod game;
mod orbit_camera;
mod physics;
mod player;

use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::new().add_plugins(GamePlugin).run();
}
