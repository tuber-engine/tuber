use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::graphics::{Graphics, GraphicsAPI, RectangleShape, Transform2D};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::keyboard::Key;
use tuber::*;

const BALL_COUNT: usize = 1;
const PADDLE_WIDTH: f32 = 20.0;
const PADDLE_HEIGHT: f32 = 100.0;
const BALL_SIZE: f32 = 10.0;
const LEFT_PADDLE_INITIAL_POSITION: (f32, f32) = (50.0, 250.0);
const RIGHT_PADDLE_INITIAL_POSITION: (f32, f32) = (730.0, 250.0);
const BALL_INITIAL_POSITION: (f32, f32) = (395.0, 295.0);

struct Ball;
struct Paddle;
struct Player;

struct Velocity {
    x: f32,
    y: f32,
}

fn main() -> tuber::Result<()> {
    let mut engine = Engine::new();

    let _left_paddle = engine.ecs().insert((
        RectangleShape {
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            color: (1.0, 1.0, 1.0),
        },
        Transform2D {
            translation: LEFT_PADDLE_INITIAL_POSITION,
        },
        Paddle,
        Player,
    ));

    let _right_paddle = engine.ecs().insert((
        RectangleShape {
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
            color: (1.0, 1.0, 1.0),
        },
        Transform2D {
            translation: RIGHT_PADDLE_INITIAL_POSITION,
        },
        Paddle,
    ));

    use rand::Rng;
    let mut rng = rand::thread_rng();
    for _ in 0..BALL_COUNT {
        let _ball = engine.ecs().insert((
            RectangleShape {
                width: BALL_SIZE,
                height: BALL_SIZE,
                color: (
                    rng.gen_range(0.0..=1.0),
                    rng.gen_range(0.0..=1.0),
                    rng.gen_range(0.0..=1.0),
                ),
            },
            Velocity {
                x: rng.gen_range(-10.0..=-5.0),
                y: rng.gen_range(-10.0..=5.0),
            },
            Transform2D {
                translation: BALL_INITIAL_POSITION,
            },
            Ball,
        ));
    }

    let mut runner = WinitTuberRunner;
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    let mut bundle = SystemBundle::new();
    bundle.add_system(move_ball_system);
    bundle.add_system(move_paddle_system);
    bundle.add_system(collision_system);
    engine.add_system_bundle(graphics.default_system_bundle());
    engine.add_system_bundle(bundle);

    runner.run(engine, graphics)
}

fn move_paddle_system(ecs: &mut Ecs) {
    let input_state = ecs.resource::<InputState>();
    for (_id, (mut transform, _)) in ecs.query::<(W<Transform2D>, R<Player>)>() {
        if input_state.is(Input::KeyDown(Key::Z)) {
            transform.translation.1 -= 5.0;
        } else if input_state.is(Input::KeyDown(Key::S)) {
            transform.translation.1 += 5.0;
        }
    }
}

fn move_ball_system(ecs: &mut Ecs) {
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

fn collision_system(ecs: &mut Ecs) {
    {
        for (_paddle_id, (paddle_transform, paddle_shape, _)) in
            ecs.query::<(R<Transform2D>, R<RectangleShape>, R<Paddle>)>()
        {
            let paddle_position = paddle_transform.translation;
            for (_ball_id, (mut ball_transform, mut velocity, _)) in
                ecs.query::<(W<Transform2D>, W<Velocity>, R<Ball>)>()
            {
                let ball_position = ball_transform.translation;

                if !ball_is_close_to_paddle(
                    ball_position,
                    BALL_SIZE,
                    paddle_transform.translation,
                    PADDLE_WIDTH,
                    PADDLE_HEIGHT,
                ) {
                    continue;
                }

                if ball_position.0 < paddle_position.0 + paddle_shape.width
                    && ball_position.0 + BALL_SIZE > paddle_position.0
                    && ball_position.1 > paddle_position.1
                    && ball_position.1 + BALL_SIZE < paddle_position.1 + paddle_shape.height
                {
                    ball_transform.translation.0 += if velocity.x >= 0.0 {
                        -(ball_position.0 + BALL_SIZE - paddle_position.0)
                    } else {
                        paddle_position.0 + paddle_shape.width - ball_position.0
                    };
                    velocity.x = -velocity.x;
                }

                if ball_position.1 < paddle_position.1 + paddle_shape.height
                    && ball_position.1 + BALL_SIZE > paddle_position.1
                    && ball_position.0 > paddle_position.0
                    && ball_position.0 + BALL_SIZE < paddle_position.0 + paddle_shape.width
                {
                    ball_transform.translation.1 += if velocity.y >= 0.0 {
                        -(ball_position.1 + BALL_SIZE - paddle_position.1)
                    } else {
                        paddle_position.1 + paddle_shape.height - ball_position.1
                    };
                    velocity.y = -velocity.y;
                }
            }
        }
    }
}

fn ball_is_close_to_paddle(
    ball_position: (f32, f32),
    ball_size: f32,
    paddle_position: (f32, f32),
    paddle_width: f32,
    paddle_height: f32,
) -> bool {
    ball_position.0 + ball_size > paddle_position.0 - ball_size
        && ball_position.0 < paddle_position.0 + paddle_width + ball_size
        && ball_position.1 + ball_size > paddle_position.1 - ball_size
        && ball_position.1 < paddle_position.1 + paddle_height + ball_size
}
