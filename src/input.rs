#![allow(clippy::mem_discriminant_non_enum)]
use crate::FromRon;

use bevy::prelude::*;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::mem::{discriminant, Discriminant};

pub struct Actions<E> {
    /// Currently active player actions
    active: HashMap<Discriminant<E>, E>,
    /// New actions of the player this frame
    new: HashMap<Discriminant<E>, E>,
    /// Action finished this frame
    finished: HashMap<Discriminant<E>, E>,
}

impl<E> Actions<E>
where
    E: Copy,
{
    pub fn update(&mut self) {
        self.new.clear();
        self.finished.clear();
    }

    pub fn start(&mut self, action: E) {
        let key = discriminant(&action);
        if !self.active.contains_key(&key) {
            self.new.insert(key, action);
        }

        self.active.insert(key, action);
    }

    pub fn stop(&mut self, action: E) {
        let key = discriminant(&action);
        if let Some(action) = self.active.remove(&key) {
            self.finished.insert(key, action);
        }
    }

    pub fn get<T: Default>(&self, action: fn(T) -> E) -> Option<&E> {
        self.active.get(&discriminant(&action(Default::default())))
    }

    pub fn just_started<T: Default>(&self, action: fn(T) -> E) -> Option<&E> {
        self.new.get(&discriminant(&action(Default::default())))
    }

    pub fn just_finished<T: Default>(&self, action: fn(T) -> E) -> Option<&E> {
        self.finished
            .get(&discriminant(&action(Default::default())))
    }
}

impl<E> Default for Actions<E> {
    fn default() -> Self {
        Self {
            active: Default::default(),
            new: Default::default(),
            finished: Default::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Action {
    Move(Vec2),
    Shoot(Vec2),
    Item,
    Bomb,
    Card,
    // Drop,
}

impl Action {
    fn is_state(&self) -> bool {
        matches!(self, Self::Move(_) | Self::Shoot(_))
    }
}

#[derive(Serialize, Deserialize)]
struct KeyBindings(HashMap<KeyCode, Action>);

impl KeyBindings {
    fn get(&self, key: &KeyCode) -> Option<Action> {
        if let Some(&action) = self.0.get(key) {
            Some(action)
        } else {
            None
        }
    }

    fn wasd() -> Self {
        use Action::*;
        Self(
            vec![
                (KeyCode::W, Move(Vec2::new(0.0, 1.0))),
                (KeyCode::A, Move(Vec2::new(-1.0, 0.0))),
                (KeyCode::S, Move(Vec2::new(0.0, -1.0))),
                (KeyCode::D, Move(Vec2::new(1.0, 0.0))),
                (KeyCode::Up, Shoot(Vec2::new(0.0, 1.0))),
                (KeyCode::Left, Shoot(Vec2::new(-1.0, 0.0))),
                (KeyCode::Down, Shoot(Vec2::new(0.0, -1.0))),
                (KeyCode::Right, Shoot(Vec2::new(1.0, 0.0))),
            ]
            .into_iter()
            .collect(),
        )
    }
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self::wasd()
    }
}

pub struct InputPlugin;

impl InputPlugin {
    // const STAGE: &'static str = "isaac_input";

    fn keyboard(
        mut actions: ResMut<Actions<Action>>,
        keys: Res<Input<KeyCode>>,
        bindings: Res<KeyBindings>,
    ) {
        actions.update();

        keys.get_just_pressed()
            .filter_map(|key| bindings.get(key))
            .filter(|action| !action.is_state())
            .for_each(|action| {
                debug!("Pressed {:?}", action);
                actions.start(action);
            });

        // updating the actions that requires several inputs to be calculated
        let mut direction = Vec2::default();
        let mut shoot_direction = Vec2::default();

        keys.get_pressed()
            .filter_map(|key| bindings.get(key))
            .filter(|action| action.is_state())
            .for_each(|action| match action {
                Action::Move(dir) => direction += dir,
                Action::Shoot(dir) => shoot_direction += dir,
                _ => (),
            });

        if direction.length_squared() > f32::EPSILON {
            actions.start(Action::Move(direction.normalize()));
        } else {
            actions.stop(Action::Move(Default::default()));
        }

        if shoot_direction.length_squared() > f32::EPSILON {
            actions.start(Action::Shoot(shoot_direction.normalize()));
        } else {
            actions.stop(Action::Shoot(Default::default()));
        }

        keys.get_just_released()
            .filter_map(|key| bindings.get(key))
            .filter(|action| !action.is_state())
            .for_each(|action| {
                debug!("Released {:?}", action);
                actions.stop(action);
            });
    }
}

impl Plugin for InputPlugin {
    fn build(&self, app: &mut AppBuilder) {
        let bindings = KeyBindings::from_file("key_bindings.ron")
            .map_err(|e| {
                println!("{}", e);
                e
            })
            .unwrap_or_default();
        app.add_resource(bindings)
            .init_resource::<Actions<Action>>()
            //.add_stage(IsaacInputs::STAGE)
            .add_system(InputPlugin::keyboard.system());
    }
}
