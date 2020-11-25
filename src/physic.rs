use bevy::prelude::*;

#[derive(Debug, Default)]
pub struct Velocity(pub Vec2);

#[derive(Debug, Default)]
pub struct Movement {
    pub direction: Option<Vec2>,
    pub speed: f32,
    pub acceleration: f32,
    pub damping: f32,
}

pub struct IsaacPhysic;

impl IsaacPhysic {
    fn moving(time: Res<Time>, mut query: Query<(&Movement, &mut Velocity)>) {
        let dt = time.delta_seconds;
        for (movement, mut velocity) in query.iter_mut() {
            let is_moving = movement.direction.is_some();
            let target = movement.speed * movement.direction.unwrap_or_default();
            let current = velocity.0;
            let error = target - current;
            // println!("Target: {:?}", target);
            let norm = error.length();
            if norm > f32::EPSILON {
                let coeff = if is_moving {
                    movement.acceleration
                } else {
                    movement.damping
                };
                velocity.0 = if norm <= coeff * dt {
                    target
                } else {
                    current + coeff * dt / norm * error
                };
            }
        }
    }

    fn physics(time: Res<Time>, mut query: Query<(Option<&Velocity>, &mut Transform)>) {
        let dt = time.delta_seconds;
        for (velocity, mut transform) in query.iter_mut() {
            if let Some(Velocity(speed)) = velocity {
                transform.translation += speed.extend(0.0) * dt;
            }
        }
    }
}

impl Plugin for IsaacPhysic {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(Self::moving.system())
            .add_system(Self::physics.system());
    }
}