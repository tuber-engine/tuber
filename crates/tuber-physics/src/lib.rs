use tuber_common::transform::Transform2D;
use tuber_core::DeltaTime;
use tuber_ecs::ecs::Ecs;
use tuber_ecs::query::accessors::W;
use tuber_ecs::system::SystemBundle;

type Vector2 = nalgebra::Vector2<f32>;

pub struct Physics {
    gravity: Vector2,
}

impl Physics {
    pub fn new(gravity: (f32, f32)) -> Self {
        Self {
            gravity: Vector2::new(gravity.0, gravity.1),
        }
    }

    pub fn update_rigid_body_2d(
        &mut self,
        delta_time: f64,
        transform: &mut Transform2D,
        rigid_body: &mut RigidBody2D,
    ) {
        rigid_body.acceleration += self.gravity;
        rigid_body.velocity = rigid_body.velocity + rigid_body.acceleration * delta_time as f32;
        transform.translation.0 += rigid_body.velocity.x;
        transform.translation.1 += rigid_body.velocity.y;
    }

    pub fn default_system_bundle() -> SystemBundle {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(physics_update_system);
        system_bundle
    }
}

pub fn physics_update_system(ecs: &mut Ecs) {
    let DeltaTime(delta_time) = *ecs
        .resource::<DeltaTime>()
        .expect("DeltaTime resource not found");
    let mut physics = ecs.resource_mut::<Physics>().expect("No Physics resource");
    for (_, (mut transform, mut rigid_body)) in ecs.query::<(W<Transform2D>, W<RigidBody2D>)>() {
        physics.update_rigid_body_2d(delta_time, &mut transform, &mut rigid_body);
    }
}

pub struct RigidBody2D {
    pub velocity: Vector2,
    pub acceleration: Vector2,
}

impl Default for RigidBody2D {
    fn default() -> Self {
        Self {
            velocity: Vector2::new(0.0, 0.0),
            acceleration: Vector2::new(0.0, 0.0),
        }
    }
}
