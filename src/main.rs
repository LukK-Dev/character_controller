#![allow(clippy::type_complexity)]

mod flycam;
mod game;
mod physics;
mod player;

use bevy::prelude::*;
use game::GamePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GamePlugin)
        .run();
}
