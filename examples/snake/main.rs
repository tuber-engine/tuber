use rand::{thread_rng, Rng};
use std::collections::VecDeque;
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::{sprite::*, transform::Transform2D, Graphics};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::keyboard::Key;
use tuber::Input::KeyDown;
use tuber::*;
use tuber::{ecs::ecs::Ecs, ecs::query::accessors::*, ecs::system::*, Result};
use tuber_core::ecs::EntityIndex;

const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
const BODY_PART_SIZE: f32 = 64.0;
const SNAKE_SPEED: f32 = 4.0;

struct SnakeHead;
struct SnakeTail;
struct SnakeBodyPart {
    next_body_part: Option<EntityIndex>,
}

#[derive(Copy, Clone)]
struct Pivot {
    position: (f32, f32),
    angle: f32,
}

struct PivotList(VecDeque<Pivot>);

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

    engine.ecs().insert((
        OrthographicCamera {
            left: 0.0,
            right: 800.0,
            top: 0.0,
            bottom: 600.0,
            near: -100.0,
            far: 100.0,
        },
        Transform2D {
            translation: (0.0, 0.0),
            ..Default::default()
        },
        Active,
    ));

    engine.ecs().insert_resource(PivotList(VecDeque::new()));
    engine.ecs().insert_resource(Score(0));

    spawn_snake(engine.ecs());
    spawn_apple(engine.ecs());

    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    let mut bundle = SystemBundle::new();
    bundle.add_system(move_head_system);
    bundle.add_system(move_body_parts_system);
    bundle.add_system(eat_apple_system);
    bundle.add_system(check_collision_with_body_system);
    engine.add_system_bundle(bundle);
    engine.add_system_bundle(graphics.default_system_bundle());
    WinitTuberRunner.run(engine, graphics)
}

fn check_collision_with_body_system(ecs: &mut Ecs) {
    let mut is_game_over = false;
    {
        let (head_id, (_, head_body_part, head_transform)) = ecs
            .query_one::<(R<SnakeHead>, R<SnakeBodyPart>, R<Transform2D>)>()
            .unwrap();
        let next_id = head_body_part.next_body_part.unwrap();
        for (body_part_id, (_, body_part_transform)) in
            ecs.query::<(R<SnakeBodyPart>, R<Transform2D>)>()
        {
            if head_id == body_part_id || next_id == body_part_id {
                continue;
            }

            if rectangle_intersects(
                (
                    head_transform.translation.0,
                    head_transform.translation.1,
                    BODY_PART_SIZE,
                    BODY_PART_SIZE,
                ),
                (
                    body_part_transform.translation.0,
                    body_part_transform.translation.1,
                    BODY_PART_SIZE,
                    BODY_PART_SIZE,
                ),
            ) {
                is_game_over = true;
            }
        }
    }

    if is_game_over {
        game_over(ecs);
    }
}

fn move_head_system(ecs: &mut Ecs) {
    let is_game_over = {
        let input_state = ecs.resource::<InputState>().unwrap();
        let (_, (_, mut velocity, mut transform)) = ecs
            .query_one::<(R<SnakeHead>, W<Velocity>, W<Transform2D>)>()
            .unwrap();

        let mut pivot_list = ecs.resource_mut::<PivotList>().unwrap();
        if input_state.is(KeyDown(Key::Q)) {
            transform.angle -= 2.0;
            pivot_list.0.push_back(Pivot {
                position: transform.translation,
                angle: transform.angle,
            });
        } else if input_state.is(KeyDown(Key::D)) {
            transform.angle += 2.0;
            pivot_list.0.push_back(Pivot {
                position: transform.translation,
                angle: transform.angle,
            });
        }

        *velocity = compute_new_segment_velocity(transform.angle);
        *transform = compute_new_segment_position(*transform, &velocity);

        transform.translation.0 < -BODY_PART_SIZE
            || transform.translation.0 > WINDOW_WIDTH as f32
            || transform.translation.1 < -BODY_PART_SIZE
            || transform.translation.1 > WINDOW_HEIGHT as f32
    };

    if is_game_over {
        game_over(ecs);
    }
}

fn game_over(ecs: &mut Ecs) {
    println!("Game Over");
    reset_score(ecs);
    respawn_snake(ecs);
}

fn reset_score(ecs: &mut Ecs) {
    let mut score = ecs.resource_mut::<Score>().unwrap();
    score.0 = 0;
    println!("Score: {}", score.0);
}

fn respawn_snake(ecs: &mut Ecs) {
    ecs.delete_by_query::<(R<SnakeBodyPart>,)>();
    spawn_snake(ecs);
}

fn spawn_apple(ecs: &mut Ecs) {
    let mut rng = thread_rng();
    let _apple = ecs.insert((
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
}

fn spawn_snake(ecs: &mut Ecs) {
    let snake_tail = ecs.insert((
        Transform2D {
            translation: (300.0, 300.0 + BODY_PART_SIZE),
            rotation_center: (32.0, BODY_PART_SIZE),
            ..Default::default()
        },
        Sprite {
            width: BODY_PART_SIZE,
            height: BODY_PART_SIZE,
            texture: "examples/snake/snake_tail.png".into(),
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeBodyPart {
            next_body_part: None,
        },
        SnakeTail,
    ));
    let _snake_head = ecs.insert((
        Transform2D {
            translation: (300.0, 300.0),
            rotation_center: (BODY_PART_SIZE / 2.0, BODY_PART_SIZE),
            ..Default::default()
        },
        Sprite {
            width: BODY_PART_SIZE,
            height: BODY_PART_SIZE,
            texture: "examples/snake/snake_face.png".into(),
        },
        Velocity {
            x: 0.0,
            y: -SNAKE_SPEED,
        },
        SnakeHead,
        SnakeBodyPart {
            next_body_part: Some(snake_tail),
        },
    ));
}

fn move_body_parts_system(ecs: &mut Ecs) {
    let (head_id, _) = ecs.query_one::<(R<SnakeHead>,)>().unwrap();
    let (tail_id, _) = ecs.query_one::<(R<SnakeTail>,)>().unwrap();
    let mut pivots = ecs.resource_mut::<PivotList>().unwrap();
    let mut pivots_to_delete = vec![];
    for (body_part_id, (mut transform, mut velocity)) in
        ecs.query::<(W<Transform2D>, W<Velocity>)>()
    {
        if body_part_id == head_id {
            continue;
        }

        for (pivot_index, pivot) in pivots.0.iter().enumerate() {
            if (transform.translation.0 - pivot.position.0).abs() < 0.2
                && (transform.translation.1 - pivot.position.1).abs() < 0.2
            {
                if body_part_id == tail_id {
                    pivots_to_delete.push(pivot_index);
                }
                transform.angle = pivot.angle;
            }
        }
        *velocity = compute_new_segment_velocity(transform.angle);
        *transform = compute_new_segment_position(*transform, &velocity);
    }

    for id_pivot_to_delete in pivots_to_delete {
        pivots.0.remove(id_pivot_to_delete);
    }
}

fn eat_apple_system(ecs: &mut Ecs) {
    let mut grow_snake = false;
    {
        let (_, (_, head_transform, head_sprite)) = ecs
            .query_one::<(R<SnakeHead>, R<Transform2D>, R<Sprite>)>()
            .unwrap();
        let mut score = ecs.resource_mut::<Score>().unwrap();
        let head_rectangle = (
            head_transform.translation.0,
            head_transform.translation.1,
            head_sprite.width,
            head_sprite.height,
        );

        let mut rng = thread_rng();
        for (_, (_, mut apple_transform, apple_sprite)) in
            ecs.query::<(R<Apple>, W<Transform2D>, R<Sprite>)>()
        {
            let apple_rectangle = (
                apple_transform.translation.0,
                apple_transform.translation.1,
                apple_sprite.width,
                apple_sprite.height,
            );

            if rectangle_intersects(head_rectangle, apple_rectangle) {
                apple_transform.translation.0 = rng.gen_range(0.0..800.0 - 64.0);
                apple_transform.translation.1 = rng.gen_range(0.0..600.0 - 64.0);
                score.0 += 1;
                grow_snake = true;
                println!("Score: {}", score.0)
            }
        }
    }

    if grow_snake {
        let (old_tail_id, tail_transform, tail_velocity) = {
            let (tail_id, (_, tail_transform, tail_velocity)) = ecs
                .query_one::<(R<SnakeTail>, R<Transform2D>, R<Velocity>)>()
                .unwrap();
            (tail_id, *tail_transform, *tail_velocity)
        };

        let new_tail_id = {
            ecs.insert((
                Transform2D {
                    translation: (
                        tail_transform.translation.0 - BODY_PART_SIZE / 4.0 * tail_velocity.x,
                        tail_transform.translation.1 - BODY_PART_SIZE / 4.0 * tail_velocity.y,
                    ),
                    ..tail_transform
                },
                Sprite {
                    width: 64.0,
                    height: 64.0,
                    texture: "examples/snake/snake_tail.png".into(),
                },
                tail_velocity,
                SnakeBodyPart {
                    next_body_part: None,
                },
                SnakeTail,
            ))
        };

        {
            {
                let (_, (mut old_tail_body_part, mut sprite)) =
                    ecs.query_one_by_id::<(W<SnakeBodyPart>, W<Sprite>)>(old_tail_id);
                old_tail_body_part.next_body_part = Some(new_tail_id);
                sprite.texture = "examples/snake/snake_body.png".into();
            }
            ecs.remove_component::<SnakeTail>(old_tail_id);
        }
    }
}

fn compute_new_segment_velocity(angle_degrees: f32) -> Velocity {
    let angle_radians = angle_degrees.to_radians();
    Velocity {
        x: SNAKE_SPEED * angle_radians.sin(),
        y: -SNAKE_SPEED * angle_radians.cos(),
    }
}

fn compute_new_segment_position(transform: Transform2D, velocity: &Velocity) -> Transform2D {
    Transform2D {
        translation: (
            transform.translation.0 + velocity.x,
            transform.translation.1 + velocity.y,
        ),
        ..transform
    }
}

fn rectangle_intersects(
    first_rectangle: (f32, f32, f32, f32),
    second_rectangle: (f32, f32, f32, f32),
) -> bool {
    return first_rectangle.0 < second_rectangle.0 + second_rectangle.2
        && first_rectangle.0 + first_rectangle.2 > second_rectangle.0
        && first_rectangle.1 < second_rectangle.1 + second_rectangle.3
        && first_rectangle.1 + first_rectangle.3 > second_rectangle.1;
}
