use crate::ecs::Ecs;

pub struct SystemBundle<'a> {
    systems: Vec<Box<dyn Fn(&'a mut Ecs)>>,
}

impl<'a> SystemBundle<'a> {
    pub fn new() -> Self {
        SystemBundle { systems: vec![] }
    }

    pub fn add_system<S: IntoSystem<'a>>(&mut self, system: S) {
        self.systems.push(system.into_system());
    }
}

pub trait IntoSystem<'a> {
    fn into_system(self) -> Box<dyn Fn(&'a mut Ecs)>;
}

impl<'a, F> IntoSystem<'a> for F
where
    F: 'static + Fn(&'a mut Ecs),
{
    fn into_system(self) -> Box<dyn Fn(&'a mut Ecs)> {
        Box::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_into_system() {
        let _ = (|_: &mut Ecs| {}).into_system();
    }

    #[test]
    fn system_bundle_add() {
        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(|_: &mut Ecs| {});
        assert_eq!(system_bundle.systems.len(), 1)
    }
}
