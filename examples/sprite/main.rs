use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::texture::{TextureRegion, TextureSource};
use tuber::graphics::{Graphics, Sprite, Transform2D};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::*;

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
        Transform2D {
            translation: (375.0, 275.0),
            ..Default::default()
        },
        Sprite {
            width: 50.0,
            height: 50.0,
            texture: "examples/sprite/sprite.png".into(),
        },
    ));

    engine.ecs().insert((
        Transform2D {
            translation: (500.0, 275.0),
            ..Default::default()
        },
        Sprite {
            width: 50.0,
            height: 50.0,
            texture: "examples/sprite/sprite2.png".into(),
        },
    ));

    engine.ecs().insert((
        Transform2D {
            translation: (250.0, 275.0),
            ..Default::default()
        },
        Sprite {
            width: 50.0,
            height: 50.0,
            texture: "fqgqgqgpng".into(),
        },
    ));

    engine.ecs().insert((
        Transform2D {
            translation: (250.0, 350.0),
            ..Default::default()
        },
        Sprite {
            width: 100.0,
            height: 100.0,
            texture: TextureSource::TextureRegion(
                "mkgskgsmlgk".into(),
                TextureRegion::new(0.0, 0.0, 16.0, 16.0),
            ),
        },
    ));

    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));
    engine.add_system_bundle(graphics.default_system_bundle());

    WinitTuberRunner.run(engine, graphics)
}
