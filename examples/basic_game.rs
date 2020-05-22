use tuber::Engine;
use tuber::GameState;

struct BasicGameState;
impl GameState for BasicGameState {
    fn initialize(&mut self) {
        println!("Initializing game state");
    }

    fn update(&mut self) {
        println!("Updating game state");
    }
}

fn main() {
    let mut engine = Engine::new("fjorgyn");
    engine.ignite(Box::new(BasicGameState)).unwrap();
}
