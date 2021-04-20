use rand;
use std::collections::HashSet;
use std::fmt;
use std::fmt::{Display, Formatter};
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::graphics::{Graphics, GraphicsAPI, RectangleShape, Transform2D};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::*;

struct Velocity {
    x: f32,
    y: f32,
}

fn move_system(ecs: &mut Ecs) {
    for (_id, (rectangle_shape, mut transform, mut velocity)) in
        ecs.query::<(R<RectangleShape>, W<Transform2D>, W<Velocity>)>()
    {
        if (transform.translation.0 + rectangle_shape.width >= 800.0)
            || (transform.translation.0 <= 0.0)
        {
            velocity.x = -velocity.x;
        }

        if (transform.translation.1 + rectangle_shape.height >= 600.0)
            || (transform.translation.1 <= 0.0)
        {
            velocity.y = -velocity.y;
        }

        transform.translation.0 += velocity.x;
        transform.translation.1 += velocity.y;
    }
}

fn log_position_system(ecs: &mut Ecs) {
    for (id, (transform_2d,)) in ecs.query::<(R<Transform2D>,)>() {
        println!("{}: {:?}", id, transform_2d);
    }
}

fn collision_system(ecs: &mut Ecs) {
    let mut collided = HashSet::new();
    {
        for (first_entity_id, (first_rectangle, first_transform)) in
            ecs.query::<(R<RectangleShape>, R<Transform2D>)>()
        {
            for (second_entity_id, (second_rectangle, second_transform)) in
                ecs.query::<(R<RectangleShape>, R<Transform2D>)>()
            {
                if first_entity_id == second_entity_id {
                    continue;
                }

                if first_transform.translation.0
                    < second_transform.translation.0 + second_rectangle.width
                    && first_transform.translation.0 + first_rectangle.width
                        > second_transform.translation.0
                    && first_transform.translation.1
                        < second_transform.translation.1 + second_rectangle.height
                    && first_transform.translation.1 + first_rectangle.height
                        > second_transform.translation.1
                {
                    collided.insert(first_entity_id);
                    collided.insert(second_entity_id);
                }
            }
        }
    }

    for (entity_id, (mut velocity,)) in ecs.query::<(W<Velocity>,)>() {
        if collided.contains(&entity_id) {
            velocity.x = -velocity.x;
            velocity.y = -velocity.y;
        }
    }
}

fn main() -> tuber::Result<()> {
    use rand::{thread_rng, Rng};
    let mut engine = Engine::new();

    let mut rng = thread_rng();
    for _ in 0..5 {
        engine.ecs().insert((
            Transform2D {
                translation: (rng.gen_range(0.0..=700.0), rng.gen_range(0.0..=500.0)),
            },
            Velocity {
                x: rng.gen_range(1.0..=5.0),
                y: rng.gen_range(1.0..=5.0),
            },
            RectangleShape {
                width: rng.gen_range(50.0..=100.0),
                height: rng.gen_range(50.0..=100.0),
                color: (
                    rng.gen_range(0.0..=1.0),
                    rng.gen_range(0.0..=1.0),
                    rng.gen_range(0.0..=1.0),
                ),
            },
        ));
    }

    let mut runner = WinitTuberRunner;
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    let mut bundle = SystemBundle::new();
    bundle.add_system(move_system);
    //bundle.add_system(log_position_system);
    bundle.add_system(collision_system);
    engine.add_system_bundle(graphics.default_system_bundle());
    engine.add_system_bundle(bundle);

    runner.run(engine, graphics)
}
