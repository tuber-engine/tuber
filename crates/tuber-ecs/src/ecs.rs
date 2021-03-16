use crate::archetype::Archetype;
use crate::component_bundle::ComponentBundle;
use crate::Entity;
use std::any::TypeId;
use std::collections::HashMap;
use tuber_core::Result;

pub type ArchetypeStore = HashMap<Box<[TypeId]>, Archetype>;

pub struct Ecs {
    archetype_store: ArchetypeStore,
    next_entity: Entity,
}

impl Ecs {
    pub fn new() -> Self {
        Self {
            archetype_store: HashMap::new(),
            next_entity: 1usize,
        }
    }

    pub fn entity_count(&self) -> usize {
        self.next_entity - 1
    }

    pub fn insert<CB: for<'a> ComponentBundle<'a>>(
        &mut self,
        component_bundles: Vec<CB>,
    ) -> Result<()> {
        for component_bundle in component_bundles.into_iter() {
            self.insert_one(component_bundle)?;
        }

        Ok(())
    }

    pub fn insert_one<CB: for<'a> ComponentBundle<'a>>(
        &mut self,
        component_bundle: CB,
    ) -> Result<Entity> {
        let entity = self.next_entity;
        let archetype = self
            .archetype_store
            .entry(CB::type_ids())
            .or_insert(Archetype::new(component_bundle.metadata()));
        let data_index = archetype.allocate_storage_for_entity(entity);
        component_bundle.write_into(archetype, data_index);
        self.next_entity += 1;
        Ok(entity)
    }

    pub fn entity<CB: for<'a> ComponentBundle<'a>>(
        &self,
        entity: Entity,
    ) -> Result<<CB as ComponentBundle<'_>>::Ref> {
        let archetype = self.archetype_store.get(&CB::type_ids()).unwrap();
        CB::read_entity(archetype, entity)
    }

    pub fn entity_mut<CB: for<'a> ComponentBundle<'a>>(
        &self,
        entity: Entity,
    ) -> Result<<CB as ComponentBundle<'_>>::RefMut> {
        let archetype = self.archetype_store.get(&CB::type_ids()).unwrap();
        CB::read_entity_mut(archetype, entity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq, Debug)]
    struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[derive(PartialEq, Debug)]
    struct Velocity {
        pub x: f32,
        pub y: f32,
    }

    #[test]
    fn ecs_new() {
        let ecs = Ecs::new();
        assert_eq!(ecs.entity_count(), 0usize);
    }

    #[test]
    fn ecs_insert_one_entity() {
        let mut ecs = Ecs::new();
        ecs.insert(vec![(
            Position { x: 2.0, y: 1.0 },
            Velocity { x: 1.5, y: 2.6 },
        )])
        .unwrap();
        assert_eq!(ecs.entity_count(), 1usize);
    }

    #[test]
    fn ecs_insert_two_entities() {
        let mut ecs = Ecs::new();
        ecs.insert(vec![
            (Position { x: 2.0, y: 1.0 }, Velocity { x: 1.5, y: 2.6 }),
            (Position { x: 0.2, y: 0.5 }, Velocity { x: 3.0, y: 2.0 }),
        ])
        .unwrap();
        assert_eq!(ecs.entity_count(), 2usize);
    }

    #[test]
    fn ecs_entity() {
        let mut ecs = Ecs::new();
        let entity = ecs
            .insert_one((Position { x: 2.0, y: 1.0 }, Velocity { x: 1.5, y: 2.6 }))
            .unwrap();

        let (position, velocity) = ecs.entity::<(Position, Velocity)>(entity).unwrap();
        assert_eq!(position, &Position { x: 2.0, y: 1.0 });
        assert_eq!(velocity, &Velocity { x: 1.5, y: 2.6 });

        let second_entity = ecs
            .insert_one((Position { x: 4.0, y: 1.0 }, Velocity { x: 1.2, y: 28.6 }))
            .unwrap();

        let (position, velocity) = ecs.entity::<(Position, Velocity)>(entity).unwrap();
        assert_eq!(position, &Position { x: 2.0, y: 1.0 });
        assert_eq!(velocity, &Velocity { x: 1.5, y: 2.6 });

        let (position, velocity) = ecs.entity::<(Position, Velocity)>(second_entity).unwrap();
        assert_eq!(position, &Position { x: 4.0, y: 1.0 });
        assert_eq!(velocity, &Velocity { x: 1.2, y: 28.6 });
    }

    #[test]
    fn ecs_entity_mut() {
        let mut ecs = Ecs::new();
        let entity = ecs
            .insert_one((Position { x: 2.0, y: 1.0 }, Velocity { x: 1.5, y: 2.6 }))
            .unwrap();

        let (position, velocity) = ecs.entity_mut::<(Position, Velocity)>(entity).unwrap();
        assert_eq!(position, &mut Position { x: 2.0, y: 1.0 });
        assert_eq!(velocity, &mut Velocity { x: 1.5, y: 2.6 });

        position.x = 0.0;
        velocity.y = 50.0;

        let (position, velocity) = ecs.entity::<(Position, Velocity)>(entity).unwrap();
        assert_eq!(position, &mut Position { x: 0.0, y: 1.0 });
        assert_eq!(velocity, &mut Velocity { x: 1.5, y: 50.0 });
    }
}
