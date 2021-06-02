use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::{Graphics, Transform2D};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::keyboard::Key;
use tuber::Input::{KeyDown, KeyUp};
use tuber::*;
use tuber::{ecs::ecs::Ecs, ecs::query::accessors::*, ecs::system::*, Result};

fn main() -> Result<()> {
    let mut engine = Engine::new();

    engine.ecs().insert((
        RectangleShape {
            width: 100.0,
            height: 100.0,
            color: (1.0, 0.0, 0.0),
        },
        Transform2D {
            translation: (400.0, 300.0),
            ..Default::default()
        },
    ));

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

    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    let mut bundle = SystemBundle::new();
    bundle.add_system(move_camera_right_system);
    engine.add_system_bundle(bundle);
    engine.add_system_bundle(graphics.default_system_bundle());
    WinitTuberRunner.run(engine, graphics)
}

fn move_camera_right_system(ecs: &mut Ecs) {
    let input_state = ecs.resource::<InputState>().unwrap();
    let (_, (_, mut transform)) = ecs
        .query_one::<(R<OrthographicCamera>, W<Transform2D>)>()
        .unwrap();

    if input_state.is(KeyDown(Key::Z)) && input_state.is(KeyUp(Key::S)) {
        transform.translation.1 -= 10.0;
    } else if input_state.is(KeyDown(Key::S)) && input_state.is(KeyUp(Key::Z)) {
        transform.translation.1 += 10.0;
    }
    if input_state.is(KeyDown(Key::Q)) && input_state.is(KeyUp(Key::D)) {
        transform.translation.0 -= 10.0;
    } else if input_state.is(KeyDown(Key::D)) && input_state.is(KeyUp(Key::Q)) {
        transform.translation.0 += 10.0;
    }
}
