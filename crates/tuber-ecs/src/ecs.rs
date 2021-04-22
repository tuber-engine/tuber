//! The ecs module defines the Ecs struct which is the main entry point of tuber-ecs

use crate::query::{Query, QueryIterator};
use crate::EntityIndex;
use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;

pub type Components = HashMap<TypeId, ComponentStore>;
pub type Resources = HashMap<TypeId, RefCell<Box<dyn Any>>>;

pub struct ComponentStore {
    pub(crate) component_data: Vec<Option<RefCell<Box<dyn Any>>>>,
    pub(crate) entities_bitset: [u64; 1024],
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            component_data: vec![None],
            entities_bitset: [0u64; 1024],
        }
    }

    pub fn with_size(size: usize) -> Self {
        let mut component_data = vec![None];
        for _ in 0..size {
            component_data.push(None);
        }

        Self {
            component_data,
            entities_bitset: [0u64; 1024],
        }
    }
}

/// The Ecs itself, stores entities and runs systems
pub struct Ecs {
    components: Components,
    shared_resources: Resources,
    next_index: EntityIndex,
}

impl Ecs {
    /// Creates a new Ecs.
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            shared_resources: HashMap::new(),
            next_index: 0,
        }
    }

    pub fn insert_resource<T: 'static>(&mut self, resource: T) {
        self.shared_resources
            .insert(TypeId::of::<T>(), RefCell::new(Box::new(resource)));
    }

    pub fn resource<T: 'static>(&self) -> Ref<T> {
        Ref::map(self.shared_resources[&TypeId::of::<T>()].borrow(), |r| {
            r.downcast_ref().unwrap()
        })
    }

    pub fn resource_mut<T: 'static>(&self) -> RefMut<T> {
        RefMut::map(
            self.shared_resources
                .get(&TypeId::of::<T>())
                .as_ref()
                .unwrap()
                .borrow_mut(),
            |r| r.downcast_mut().unwrap(),
        )
    }

    /// Inserts an entity into the Ecs.
    ///
    /// This method takes an [`EntityDefinition`] describing the entity.
    ///
    /// It returns the [`EntityIndex`] of the inserted entity.
    pub fn insert<ED: EntityDefinition>(&mut self, entity_definition: ED) -> EntityIndex {
        let index = self.next_index;
        entity_definition.store_components(&mut self.components, index);
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
    fn store_components(self, components: &mut Components, index: usize);
}

macro_rules! impl_entity_definition_tuples {
    ($($t:tt => $i:tt,)*) => {
        impl<$($t: 'static,)*> EntityDefinition for ($($t,)*) {
            fn store_components(self, components: &mut Components, index: usize) {
                use crate::bitset::BitSet;

                for component_storage in components.values_mut() {
                    component_storage.component_data.push(None);
                }

                $(
                    let component_storage = components.entry(TypeId::of::<$t>()).or_insert(ComponentStore::with_size(index));
                    *component_storage.component_data.last_mut().unwrap() = (Some(RefCell::new(Box::new(self.$i))));
                    component_storage.entities_bitset.set_bit(index);
                )*
            }
        }
    }
}

impl_entity_definition_tuples!(A => 0,);
impl_entity_definition_tuples!(A => 0, B => 1,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6,);
impl_entity_definition_tuples!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5, G => 6, H => 7,);

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
        ecs.insert((Position { x: 4.0, y: 5.0 },));

        for (_, (mut velocity,)) in ecs.query::<(W<Velocity>,)>() {
            velocity.x = 0.0;
        }

        for (_, (position, velocity)) in ecs.query::<(R<Position>, R<Velocity>)>() {
            assert_ne!(position.x, 0.0);
            assert_ne!(position.y, 0.0);
            assert_eq!(velocity.x, 0.0);
            assert_ne!(velocity.y, 0.0);
        }

        assert_eq!(ecs.query::<(R<Position>,)>().count(), 3);
        assert_eq!(ecs.query::<(R<Velocity>,)>().count(), 2);
    }
}
