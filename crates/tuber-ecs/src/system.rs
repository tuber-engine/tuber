use crate::ecs::Ecs;

pub struct SystemBundle {
    systems: Vec<Box<dyn FnMut(&mut Ecs)>>,
}

impl SystemBundle {
    pub fn new() -> Self {
        SystemBundle { systems: vec![] }
    }

    pub fn add_system<S: IntoSystem>(&mut self, system: S) {
        self.systems.push(system.into_system());
    }

    pub fn step(&mut self, ecs: &mut Ecs) {
        for system in &mut self.systems {
            (system)(ecs);
        }
    }
}

pub trait IntoSystem {
    fn into_system(self) -> Box<dyn FnMut(&mut Ecs)>;
}

impl<F> IntoSystem for F
where
    F: 'static + FnMut(&mut Ecs),
{
    fn into_system(self) -> Box<dyn FnMut(&mut Ecs)> {
        Box::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::accessors::{R, W};

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

    #[test]
    fn system_bundle_step() {
        #[derive(PartialEq, Debug)]
        struct Value(i32);
        struct OtherComponent;

        let mut ecs = Ecs::new();
        ecs.insert((Value(12),));
        ecs.insert((Value(18), OtherComponent));

        let mut system_bundle = SystemBundle::new();
        system_bundle.add_system(|ecs: &mut Ecs| {
            for (_, (mut v,)) in ecs.query::<(W<Value>,)>() {
                v.0 += 35;
            }
        });
        system_bundle.add_system(|ecs: &mut Ecs| {
            for (_, (mut v,)) in ecs.query::<(W<Value>,)>() {
                v.0 -= 6;
            }
        });

        system_bundle.step(&mut ecs);
        let mut query_result = ecs.query::<(R<Value>,)>();
        assert_eq!(*query_result.next().unwrap().1.0, Value(41));
        assert_eq!(*query_result.next().unwrap().1.0, Value(47));
    }
}
