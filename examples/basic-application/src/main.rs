use tuber::Engine;
use tuber::GameStateBuilder;
use tuber::SystemBundleBuilder;

fn print_hello_world() {
    println!("Hello world");
}

fn main() -> tuber::Result<()> {
    let mut engine = Engine::new();
    let system_bundle = SystemBundleBuilder::new()
        .with_system(Box::new(|| print_hello_world()))
        .with_system(Box::new(|| println!("Test")))
        .build();
    let initial_game_state = GameStateBuilder::new()
        .with_system_bundle(system_bundle)
        .build();
    engine.push_state(initial_game_state);
    engine.ignite()
}
