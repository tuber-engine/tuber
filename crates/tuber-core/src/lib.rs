pub struct Engine;

impl Engine {
    pub fn new() -> Engine {
        Self
    }

    pub fn ignite(self) -> Result<()> {
        Ok(())
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {}

pub trait TuberRunner {
    fn run(&mut self, engine: Engine) -> Result<()>;
}
