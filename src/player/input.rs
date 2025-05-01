use super::types::Player;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

#[derive(Debug, InputAction)]
#[input_action(output = Vec2)]
pub(super) struct Move;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct Sprint;

#[derive(Debug, InputAction)]
#[input_action(output = bool)]
pub(super) struct Jump;

pub(super) fn binding(trigger: Trigger<Binding<Player>>, mut players: Query<&mut Actions<Player>>) {
    let mut actions = players.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Move>()
        .to((Cardinal::wasd_keys(), Axial::left_stick()))
        .with_modifiers(DeadZone::default());

    actions
        .bind::<Sprint>()
        .to((KeyCode::ShiftLeft, GamepadButton::South));

    actions
        .bind::<Jump>()
        .to((KeyCode::Space, GamepadButton::East))
        .with_conditions(JustPress::default());
}
