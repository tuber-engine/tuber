use tuber::*;

fn main() -> tuber::Result<()> {
    let engine = Engine::new();
    let mut runner = WinitTuberRunner;

    runner.run(engine)
}
