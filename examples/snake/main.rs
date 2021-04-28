use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use tuber::graphics::{Graphics, GraphicsAPI, RectangleShape, Sprite, Transform2D};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::keyboard::Key;
use tuber::Input::KeyDown;
use tuber::*;
use tuber::{ecs::ecs::Ecs, ecs::query::accessors::*, ecs::system::*, Result};
use tuber_core::ecs::EntityIndex;

const SNAKE_SPEED: f32 = 2.0;

struct SnakeHead;
struct SnakeTail;
struct SnakeBodyPart {
    pivots: VecDeque<Pivot>,
    next_body_part: Option<EntityIndex>,
}

#[derive(Copy, Clone)]
struct Pivot {
    position: (f32, f32),
    angle: f32,
}

struct Apple;

#[derive(Debug, Copy, Clone)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Debug)]
struct Score(u32);

fn main() -> Result<()> {
    let mut engine = Engine::new();

    engine.ecs().insert_resource(Score(0));
    let snake_tail = engine.ecs().insert((
        Transform2D {
            translation: (300.0, 300.0 + 128.0),
            rotation_center: (32.0, 64.0),
            ..Default::default()
        },
        Sprite {
            width: 64.0,
            height: 64.0,
            texture: "examples/snake/snake_tail.png".into(),
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeBodyPart {
            pivots: VecDeque::new(),
            next_body_part: None,
        },
        SnakeTail,
    ));
    let snake_body = engine.ecs().insert((
        Transform2D {
            translation: (300.0, 300.0 + 64.0),
            rotation_center: (32.0, 64.0),
            ..Default::default()
        },
        Sprite {
            width: 64.0,
            height: 64.0,
            texture: "examples/snake/snake_body.png".into(),
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeBodyPart {
            pivots: VecDeque::new(),
            next_body_part: Some(snake_tail),
        },
    ));
    let _snake_head = engine.ecs().insert((
        Transform2D {
            translation: (300.0, 300.0),
            rotation_center: (32.0, 64.0),
            ..Default::default()
        },
        Sprite {
            width: 64.0,
            height: 64.0,
            texture: "examples/snake/snake_face.png".into(),
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeHead,
        SnakeBodyPart {
            pivots: VecDeque::new(),
            next_body_part: Some(snake_body),
        },
    ));

    let mut rng = thread_rng();
    let _apple = engine.ecs().insert((
        Transform2D {
            translation: (
                rng.gen_range(0.0..800.0 - 64.0),
                rng.gen_range(0.0..600.0 - 64.0),
            ),
            ..Default::default()
        },
        Sprite {
            width: 64.0,
            height: 64.0,
            texture: "examples/snake/apple.png".into(),
        },
        Apple,
    ));

    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    let mut bundle = SystemBundle::new();
    bundle.add_system(move_player_system);
    bundle.add_system(collide_apple_system);
    engine.add_system_bundle(bundle);
    engine.add_system_bundle(graphics.default_system_bundle());
    WinitTuberRunner.run(engine, graphics)
}

fn move_player_system(ecs: &mut Ecs) {
    let mut pivots_to_add = vec![];
    let mut head_id = 0;
    {
        let input_state = ecs.resource::<InputState>();
        {
            let (id, (_, body_part, mut velocity, mut transform)) = ecs
                .query_one::<(R<SnakeHead>, R<SnakeBodyPart>, W<Velocity>, W<Transform2D>)>()
                .unwrap();
            head_id = id;

            if input_state.is(KeyDown(Key::Q)) {
                transform.angle -= 1.0;

                pivots_to_add.push(Pivot {
                    position: (transform.translation.0, transform.translation.1),
                    angle: transform.angle,
                });
            } else if input_state.is(KeyDown(Key::D)) {
                transform.angle += 1.0;

                pivots_to_add.push(Pivot {
                    position: (transform.translation.0, transform.translation.1),
                    angle: transform.angle,
                });
            }

            let angle_radians = transform.angle.to_radians();
            velocity.x = SNAKE_SPEED * angle_radians.sin();
            velocity.y = -SNAKE_SPEED * angle_radians.cos();
            transform.translation.0 += velocity.x;
            transform.translation.1 += velocity.y;
        }
    }

    pivots_to_add
        .iter()
        .for_each(|pivot| add_pivot_to_each_parts(ecs, head_id, *pivot));

    for (id, (mut body_part, mut velocity, mut transform)) in
        ecs.query::<(W<SnakeBodyPart>, W<Velocity>, W<Transform2D>)>()
    {
        if id == head_id {
            continue;
        }

        if let Some(p) = body_part.pivots.front() {
            if transform.translation.0 as i32 == p.position.0 as i32
                && transform.translation.1 as i32 == p.position.1 as i32
            {
                transform.angle = p.angle;
                body_part.pivots.pop_front();
            }
        }
        let angle_radians = transform.angle.to_radians();
        velocity.x = SNAKE_SPEED * angle_radians.sin();
        velocity.y = -SNAKE_SPEED * angle_radians.cos();
        transform.translation.0 += velocity.x;
        transform.translation.1 += velocity.y;
    }
}

fn add_pivot_to_each_parts(ecs: &mut Ecs, head_id: EntityIndex, pivot: Pivot) {
    for (body_part_id, (mut body_part,)) in ecs.query::<(W<SnakeBodyPart>,)>() {
        if body_part_id == head_id {
            continue;
        }

        body_part.pivots.push_back(pivot);
    }
}

fn collide_apple_system(ecs: &mut Ecs) {
    let mut colliding = false;
    {
        let (_, (_, snake_head_transform, snake_head_sprite)) = ecs
            .query_one::<(R<SnakeHead>, R<Transform2D>, R<Sprite>)>()
            .unwrap();
        let mut rng = thread_rng();
        for (_, (_, mut apple_transform, apple_sprite)) in
            ecs.query::<(R<Apple>, W<Transform2D>, R<Sprite>)>()
        {
            if rectangle_collides(
                (
                    snake_head_transform.translation.0,
                    snake_head_transform.translation.1,
                    snake_head_sprite.width,
                    snake_head_sprite.height,
                ),
                (
                    apple_transform.translation.0,
                    apple_transform.translation.1,
                    apple_sprite.width,
                    apple_sprite.height,
                ),
            ) {
                colliding = true;
                apple_transform.translation.0 = rng.gen_range(0.0..800.0 - 64.0);
                apple_transform.translation.1 = rng.gen_range(0.0..600.0 - 64.0);

                let mut score = ecs.resource_mut::<Score>();
                score.0 += 1;
                println!("Score: {}", score.0);
            }
        }
    }

    let mut old_tail_to_delete = None;
    if colliding {
        let new_body_part = ecs.insert((
            Sprite {
                width: 64.0,
                height: 64.0,
                texture: "examples/snake/snake_tail.png".to_string(),
            },
            Transform2D {
                translation: (0.0, 0.0),
                angle: 0.0,
                rotation_center: (0.0, 0.0),
            },
            Velocity { x: 0.0, y: 0.0 },
            SnakeBodyPart {
                pivots: VecDeque::new(),
                next_body_part: None,
            },
            SnakeTail,
        ));

        let (old_tail, (_, mut tail_body_part, tail_transform, tail_velocity)) = ecs
            .query_one::<(R<SnakeTail>, W<SnakeBodyPart>, R<Transform2D>, R<Velocity>)>()
            .unwrap();
        let (new_tail, (mut transform, mut velocity, mut body_part)) =
            ecs.query_one_by_id::<(W<Transform2D>, W<Velocity>, W<SnakeBodyPart>)>(new_body_part);
        *transform = *tail_transform;
        *velocity = *tail_velocity;

        tail_body_part.next_body_part = Some(new_tail);
        old_tail_to_delete = Some(old_tail);
    }

    if let Some(old_tail_to_delete) = old_tail_to_delete {
        ecs.remove_component::<SnakeTail>(old_tail_to_delete);
    }
}

fn rectangle_collides(
    first_rectangle: (f32, f32, f32, f32),
    second_rectangle: (f32, f32, f32, f32),
) -> bool {
    return first_rectangle.0 < second_rectangle.0 + second_rectangle.2
        && first_rectangle.0 + first_rectangle.2 > second_rectangle.0
        && first_rectangle.1 < second_rectangle.1 + second_rectangle.3
        && first_rectangle.1 + first_rectangle.3 > second_rectangle.1;
}
