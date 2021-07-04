use tuber::common::transform::Transform2D;
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::physics::{CollisionShape, CollisionShapes, Physics, RigidBody2D, StaticBody2D};
use tuber::Input::MouseButtonDown;
use tuber::{Engine, InputState, TuberRunner, WinitTuberRunner};
use tuber_core::ecs::ecs::Ecs;
use tuber_core::ecs::query::accessors::{R, W};
use tuber_core::ecs::system::SystemBundle;

struct MouseControlled;

fn main() -> tuber::Result<()> {
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

    engine.ecs().insert((
        RectangleShape {
            width: 100.0,
            height: 100.0,
            color: (1.0, 0.0, 0.0),
        },
        Transform2D {
            translation: (200.0, 200.0),
            ..Default::default()
        },
        StaticBody2D,
        CollisionShapes {
            shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 100.0, 100.0)],
        },
    ));

    engine.ecs().insert((
        MouseControlled,
        RectangleShape {
            width: 100.0,
            height: 100.0,
            color: (1.0, 0.0, 0.0),
        },
        Transform2D {
            translation: (200.0, 0.0),
            ..Default::default()
        },
        RigidBody2D::default(),
        CollisionShapes {
            shapes: vec![CollisionShape::from_rectangle(0.0, 0.0, 100.0, 100.0)],
        },
    ));

    let mut runner = WinitTuberRunner;
    let graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    let physics = Physics::new((0.0, 1.0));
    engine.ecs().insert_shared_resource(physics);

    engine.add_system_bundle(Physics::default_system_bundle());
    engine.add_system_bundle(Graphics::default_system_bundle());
    let mut bundle = SystemBundle::new();
    engine.add_system_bundle(bundle);

    runner.run(engine, graphics)
}
