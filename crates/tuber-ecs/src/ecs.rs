//! The ecs module defines the Ecs struct which is the main entry point of tuber-ecs

use crate::query::{Query, QueryIterator};
use crate::EntityIndex;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

pub type Components = HashMap<TypeId, Vec<Option<RefCell<Box<dyn Any>>>>>;

/// The Ecs itself, stores entities and runs systems
pub struct Ecs {
    components: Components,
    next_index: EntityIndex,
}

impl Ecs {
    /// Creates a new Ecs.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            next_index: 0,
        }
    }

    /// Inserts an entity into the Ecs.
    ///
    /// This method takes an [`EntityDefinition`] describing the entity.
    ///
    /// It returns the [`EntityIndex`] of the inserted entity.
    pub fn insert<ED: EntityDefinition>(&mut self, entity_definition: ED) -> EntityIndex {
        entity_definition.store_components(&mut self.components);
        let index = self.next_index;
        self.next_index += 1;
        index
    }

    pub fn query<'a, Q: Query<'a>>(&self) -> QueryIterator<Q> {
        QueryIterator::new(self.entity_count(), &self.components)
    }

    /// Returns the entity count of the Ecs.
    pub fn entity_count(&self) -> usize {
        self.next_index
    }
}

/// A type that can be used to define an entity
pub trait EntityDefinition {
    fn store_components(self, components: &mut Components);
}

impl<A: 'static, B: 'static> EntityDefinition for (A, B) {
    fn store_components(self, components: &mut Components) {
        let component_storage = components.entry(TypeId::of::<A>()).or_insert(vec![]);
        component_storage.push(Some(RefCell::new(Box::new(self.0))));
        let component_storage = components.entry(TypeId::of::<B>()).or_insert(vec![]);
        component_storage.push(Some(RefCell::new(Box::new(self.1))));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::accessors::*;

    #[derive(Debug)]
    struct Position {
        x: f32,
        y: f32,
    }

    #[derive(Debug)]
    struct Velocity {
        x: f32,
        y: f32,
    }

    #[test]
    pub fn ecs_new() {
        let ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0usize);
    }

    #[test]
    pub fn ecs_insert() {
        let mut ecs = Ecs::new();
        ecs.insert((Position { x: 0.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));
        assert_eq!(ecs.entity_count(), 1usize);
        ecs.insert((Position { x: 4.0, y: 5.0 }, Velocity { x: 6.0, y: 7.0 }));
        assert_eq!(ecs.entity_count(), 2usize);
    }

    #[test]
    pub fn ecs_query() {
        let mut ecs = Ecs::new();
        ecs.insert((Position { x: 12.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));
        ecs.insert((Position { x: 4.0, y: 5.0 }, Velocity { x: 6.0, y: 7.0 }));

        for (mut velocity,) in ecs.query::<(W<Velocity>,)>() {
            velocity.x = 0.0;
        }

        for (position, velocity) in ecs.query::<(R<Position>, R<Velocity>)>() {
            assert_ne!(position.x, 0.0);
            assert_ne!(position.y, 0.0);
            assert_eq!(velocity.x, 0.0);
            assert_ne!(velocity.y, 0.0);
        }
    }
}
