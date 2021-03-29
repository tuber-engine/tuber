//! The ecs module defines the Ecs struct which is the main entry point of tuber-ecs

use crate::EntityIndex;
use std::any::{Any, TypeId};
use std::collections::HashMap;

/// The Ecs itself, stores entities and runs systems
pub struct Ecs;

impl Ecs {
    /// Creates a new Ecs.
    pub fn new() -> Self {
        Self
    }

    /// Inserts an entity into the Ecs.
    ///
    /// This method takes an [`EntityDefinition`] describing the entity.
    ///
    /// It returns the [`EntityIndex`] of the inserted entity.
    pub fn insert<ED: EntityDefinition>(&mut self, entity_definition: ED) -> EntityIndex {
        unimplemented!()
    }

    /// Returns the entity count of the Ecs.
    pub fn entity_count(&self) -> usize {
        0usize
    }
}

/// A type that can be used to define an entity
pub trait EntityDefinition {}
impl<A, B> EntityDefinition for (A, B) {}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        x: f32,
        y: f32,
    }

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
    }
}
