use tuber_ecs::ecs::Ecs;
use tuber_ecs::system::SystemBundle;

pub fn default_graphics_system_bundle() -> SystemBundle {
    let mut bundle = SystemBundle::new();
    bundle.add_system(log_rendering);
    bundle
}

pub trait GraphicsAPI {
    fn render_scene();
}

fn log_rendering(_ecs: &mut Ecs) {
    println!("rendering");
}
