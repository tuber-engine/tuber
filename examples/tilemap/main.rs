use std::collections::HashSet;
use tuber::ecs::ecs::Ecs;
use tuber::ecs::query::accessors::{R, W};
use tuber::ecs::system::SystemBundle;
use tuber::graphics::camera::{Active, OrthographicCamera};
use tuber::graphics::tilemap::TilemapRender;
use tuber::graphics::{transform::Transform2D, Graphics};
use tuber::graphics_wgpu::GraphicsWGPU;
use tuber::keyboard::Key;
use tuber::Input::{KeyDown, KeyUp};
use tuber::*;
use tuber_common::tilemap::{Tile, Tilemap};

struct MapUpdateTimer(std::time::Instant);

fn main() -> tuber::Result<()> {
    let mut engine = Engine::new();

    engine.ecs().insert((
        OrthographicCamera {
            left: 0.0,
            right: 200.0,
            top: 0.0,
            bottom: 150.0,
            near: -100.0,
            far: 100.0,
        },
        Transform2D {
            translation: (0.0, 0.0),
            ..Default::default()
        },
        Active,
    ));

    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut tilemap = Tilemap::new(100, 100, 16, 16, &["dirt"]);
    for tile in &mut tilemap.tiles {
        let tile_tag = rng.gen_range(0..=2);
        let mut tags = HashSet::new();

        match tile_tag {
            0 => tags.insert("water".to_owned()),
            1 => tags.insert("sand".to_owned()),
            2 => tags.insert("dirt".to_owned()),
            _ => panic!(),
        };

        tile.tags = tags;
    }

    engine.ecs().insert((
        tilemap,
        TilemapRender {
            identifier: "tilemap".into(),
            texture_atlas_identifier: "examples/tilemap/tiles.json".to_string(),
            tile_texture_function: Box::new(|tile: &Tile| {
                if tile.tags.contains(&String::from("water")) {
                    return Some("water");
                } else if tile.tags.contains(&String::from("dirt")) {
                    return Some("dirt");
                } else if tile.tags.contains(&String::from("sand")) {
                    return Some("sand");
                }

                return None;
            }),
            dirty: true,
        },
    ));

    engine
        .ecs()
        .insert_resource(MapUpdateTimer(std::time::Instant::now()));

    let mut runner = WinitTuberRunner;
    let mut graphics = Graphics::new(Box::new(GraphicsWGPU::new()));

    let mut bundle = SystemBundle::new();
    bundle.add_system(move_camera_system);
    engine.add_system_bundle(graphics.default_system_bundle());
    engine.add_system_bundle(bundle);

    runner.run(engine, graphics)
}

fn move_camera_system(ecs: &mut Ecs) {
    let input_state = ecs.resource::<InputState>().unwrap();
    let (_, (_, mut transform)) = ecs
        .query_one::<(R<OrthographicCamera>, W<Transform2D>)>()
        .unwrap();

    if input_state.is(KeyDown(Key::Z)) && input_state.is(KeyUp(Key::S)) {
        transform.translation.1 -= 1.0;
    } else if input_state.is(KeyDown(Key::S)) && input_state.is(KeyUp(Key::Z)) {
        transform.translation.1 += 1.0;
    }

    if input_state.is(KeyDown(Key::Q)) && input_state.is(KeyUp(Key::D)) {
        transform.translation.0 -= 1.0;
    } else if input_state.is(KeyDown(Key::D)) && input_state.is(KeyUp(Key::Q)) {
        transform.translation.0 += 1.0;
    }

    if input_state.is(KeyDown(Key::A)) && input_state.is(KeyUp(Key::E)) {
        transform.scale += 0.01;
    } else if input_state.is(KeyDown(Key::E)) && input_state.is(KeyUp(Key::A)) {
        transform.scale -= 0.01;
    }
}
