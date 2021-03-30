//! The ecs module defines the Ecs struct which is the main entry point of tuber-ecs

use crate::ecs::accessors::Accessor;
use crate::EntityIndex;
use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

type Components = HashMap<TypeId, Vec<Option<RefCell<Box<dyn Any>>>>>;

mod accessors {
    use crate::ecs::Components;
    use std::any::TypeId;
    use std::cell::{Ref, RefMut};
    use std::marker::PhantomData;

    pub struct R<T>(PhantomData<T>);
    pub struct W<T>(PhantomData<T>);

    pub trait Accessor<'a> {
        type RawType: 'a;
        type RefType: 'a;

        fn fetch(index: usize, components: &'a Components) -> Self::RefType;
    }
    impl<'a, T: 'static> Accessor<'a> for R<T> {
        type RawType = T;
        type RefType = Ref<'a, T>;

        fn fetch(index: usize, components: &'a Components) -> Self::RefType {
            Ref::map(
                components[&TypeId::of::<T>()][index]
                    .as_ref()
                    .unwrap()
                    .borrow(),
                |r| r.downcast_ref().unwrap(),
            )
        }
    }
    impl<'a, T: 'static> Accessor<'a> for W<T> {
        type RawType = T;
        type RefType = RefMut<'a, T>;

        fn fetch(index: usize, components: &'a Components) -> Self::RefType {
            RefMut::map(
                components[&TypeId::of::<T>()][index]
                    .as_ref()
                    .unwrap()
                    .borrow_mut(),
                |r| r.downcast_mut().unwrap(),
            )
        }
    }
}

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

pub trait Query<'a> {
    type ResultType: 'a;

    fn fetch(index: EntityIndex, components: &'a Components) -> Self::ResultType;
}
impl<'a, A, B> Query<'a> for (A, B)
where
    A: Accessor<'a>,
    B: Accessor<'a>,
{
    type ResultType = (A::RefType, B::RefType);

    fn fetch(index: usize, components: &'a Components) -> Self::ResultType {
        (A::fetch(index, components), B::fetch(index, components))
    }
}

pub struct QueryIterator<'a, Q> {
    index: EntityIndex,
    entity_count: EntityIndex,
    components: &'a Components,
    marker: PhantomData<&'a Q>,
}

impl<'a, Q> QueryIterator<'a, Q> {
    pub fn new(entity_count: usize, components: &'a Components) -> Self {
        Self {
            index: 0,
            entity_count,
            components,
            marker: PhantomData,
        }
    }
}

impl<'a, Q> Iterator for QueryIterator<'a, Q>
where
    Q: Query<'a>,
{
    type Item = Q::ResultType;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entity_count {
            return None;
        }

        let index = self.index;
        self.index += 1;
        Some(Q::fetch(index, self.components))
    }
}

#[cfg(test)]
mod tests {
    use super::accessors::*;
    use super::*;

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
        ecs.insert((Position { x: 0.0, y: 1.0 }, Velocity { x: 2.0, y: 3.0 }));
        ecs.insert((Position { x: 4.0, y: 5.0 }, Velocity { x: 6.0, y: 7.0 }));

        for (mut position, mut velocity) in ecs.query::<(W<Position>, W<Velocity>)>() {
            position.x = 0.0;
            position.y = 0.0;
            velocity.x = 0.0;
        }

        for (position, velocity) in ecs.query::<(R<Position>, R<Velocity>)>() {
            dbg!(position);
            dbg!(velocity);
        }
    }
}
