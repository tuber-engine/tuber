use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::shape::RectangleShape;
use tuber::graphics::ui::{Frame, NoViewTransform, Text};
use tuber::graphics::Graphics;
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::keyboard::Key;
use tuber::Input::{KeyDown, KeyUp};
use tuber::*;
use tuber::{ecs::ecs::Ecs, ecs::query::accessors::*, ecs::system::*, Result};
use tuber_common::transform::Transform2D;

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

    engine.ecs().insert((
        RectangleShape {
            width: 100.0,
            height: 100.0,
            color: (0.0, 0.0, 1.0),
        },
        Transform2D {
            translation: (100.0, 100.0),
            ..Default::default()
        },
    ));

    engine.ecs().insert((
        RectangleShape {
            width: 100.0,
            height: 100.0,
            color: (0.0, 1.0, 1.0),
        },
        Transform2D {
            translation: (200.0, 200.0),
            ..Default::default()
        },
    ));

    engine.ecs().insert((
        Text::new("Health".into(), "examples/ui/font.json".into()),
        Transform2D {
            translation: (0.0, 35.0),
            ..Default::default()
        },
        NoViewTransform,
    ));

    engine.ecs().insert((
        Frame {
            width: 200.0,
            height: 50.0,
            color: (1.0, 0.0, 0.0),
        },
        Transform2D {
            translation: (75.0, 0.0),
            ..Default::default()
        },
        NoViewTransform,
    ));

    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    graphics.set_clear_color((1.0, 1.0, 1.0));
    let mut bundle = SystemBundle::new();
    bundle.add_system(move_camera_right_system);
    bundle.add_system(lose_health_system);
    engine.add_system_bundle(Graphics::default_system_bundle());
    engine.add_system_bundle(bundle);
    WinitTuberRunner.run(engine, graphics)
}

fn lose_health_system(ecs: &mut Ecs) {
    for (_, (mut frame,)) in ecs.query::<(W<Frame>,)>() {
        frame.width -= 1.0;
        if frame.width < 0.0 {
            frame.width = 200.0;
        }
    }
}

fn move_camera_right_system(ecs: &mut Ecs) {
    let input_state = ecs.shared_resource::<InputState>().unwrap();
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
