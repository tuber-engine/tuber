use std::any::TypeId;
use std::collections::HashMap;
use std::ptr::NonNull;
use tuber_core::Result;

pub type Entity = usize;
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

pub struct Archetype {
    data: NonNull<u8>,
    capacity: usize,
    entity_count: usize,
    size: usize,
    types_metadata: Vec<TypeMetadata>,
    type_offsets: Vec<usize>,
    stored_entities: HashMap<Entity, usize>,
}
impl Archetype {
    pub fn new(types_metadata: Vec<TypeMetadata>) -> Self {
        let length = types_metadata.len();
        Self {
            data: NonNull::dangling(),
            capacity: 0usize,
            entity_count: 0usize,
            size: 0usize,
            types_metadata,
            type_offsets: vec![0; length],
            stored_entities: HashMap::new(),
        }
    }

    pub fn allocate_storage_for_entity(&mut self, entity: Entity) -> usize {
        if self.entity_count == self.capacity {
            if self.capacity == 0 {
                self.grow(1);
            } else {
                self.grow(self.capacity << 1);
            }
        }

        self.stored_entities.insert(entity, self.entity_count);
        self.entity_count += 1;
        self.entity_count - 1
    }

    pub fn data_index_for_entity(&self, entity: Entity) -> usize {
        self.stored_entities[&entity]
    }

    fn grow(&mut self, new_capacity: usize) {
        use std::alloc::{alloc, dealloc, Layout};

        let (computed_size, type_offsets) = self.compute_required_size_for_capacity(new_capacity);
        let new_data;
        let alignment = self.data_alignment();
        unsafe {
            new_data = NonNull::new(alloc(
                Layout::from_size_align(computed_size, alignment).unwrap(),
            ))
            .unwrap();

            if self.capacity != 0 {
                for (type_index, type_metadata) in self.types_metadata.iter().enumerate() {
                    let old_offset = self.type_offsets[type_index];
                    let new_offset = type_offsets[type_index];
                    std::ptr::copy_nonoverlapping(
                        self.data.as_ptr().add(old_offset),
                        new_data.as_ptr().add(new_offset),
                        self.entity_count * type_metadata.layout.size(),
                    );
                }
                dealloc(
                    self.data.as_ptr(),
                    Layout::from_size_align(self.size, alignment).unwrap(),
                )
            }
        }

        self.data = new_data;
        self.type_offsets = type_offsets;
        self.capacity = new_capacity;
        self.size = computed_size;
    }

    fn compute_required_size_for_capacity(&mut self, capacity: usize) -> (usize, Vec<usize>) {
        let mut size = 0;
        let mut type_offsets = vec![0; self.types_metadata.len()];
        for (i, type_metadata) in self.types_metadata.iter().enumerate() {
            size = align_value(size, type_metadata.layout.align());
            type_offsets[i] = size;
            size += type_metadata.layout.size() * capacity;
        }
        (size, type_offsets)
    }

    pub fn write_component(
        &mut self,
        type_index: usize,
        data_index: usize,
        data_size: usize,
        data: *const u8,
    ) {
        let ptr = self.component_ptr(data_index, data_size, type_index);
        unsafe { std::ptr::copy(data, ptr, data_size) }
    }

    pub fn read_component<C>(&self, type_index: usize, data_index: usize, data_size: usize) -> &C {
        let ptr = self.component_ptr(data_index, data_size, type_index);
        unsafe { (ptr as *const C).as_ref().unwrap() }
    }

    pub fn read_component_mut<C>(
        &self,
        type_index: usize,
        data_index: usize,
        data_size: usize,
    ) -> &mut C {
        let ptr = self.component_ptr(data_index, data_size, type_index);
        unsafe { (ptr as *mut C).as_mut().unwrap() }
    }

    fn component_ptr(&self, data_index: usize, data_size: usize, type_index: usize) -> *mut u8 {
        let type_offset = self.type_offsets[type_index];

        unsafe {
            self.data
                .as_ptr()
                .add(type_offset + data_index * data_size)
                .cast::<u8>()
        }
    }

    fn data_alignment(&self) -> usize {
        self.types_metadata
            .first()
            .map_or(1, |tm| tm.layout.align())
    }
}

impl Drop for Archetype {
    fn drop(&mut self) {
        use std::alloc::{dealloc, Layout};
        if self.size > 0 {
            unsafe {
                dealloc(
                    self.data.as_ptr(),
                    Layout::from_size_align(self.size, self.data_alignment()).unwrap(),
                );
            }
        }
    }
}

pub trait ComponentBundle<'a> {
    type Ref;
    type RefMut;
    fn type_ids() -> Box<[TypeId]>;
    fn write_into(&self, archetype: &mut Archetype, data_index: usize);
    fn metadata(&self) -> Vec<TypeMetadata>;
    fn read_entity(archetype: &'a Archetype, entity: Entity) -> Result<Self::Ref>;
    fn read_entity_mut(archetype: &'a Archetype, entity: Entity) -> Result<Self::RefMut>;
}

macro_rules! impl_component_bundle {
    ($($type:ident => $index:tt,)*) => {
        impl<'a, $($type: 'static),*> ComponentBundle<'a> for ($($type,)*) {
            type Ref = ($(&'a $type,)*);
            type RefMut = ($(&'a mut $type,)*);

            fn type_ids() -> Box<[TypeId]> {
                Box::new([$(TypeId::of::<$type>(),)*])
            }

            fn write_into(&self, archetype: &mut Archetype, data_index: usize) {
                $(archetype.write_component(
                    $index,
                    data_index,
                    std::mem::size_of::<$type>(),
                    (&self.$index) as *const $type as *const u8,
                );)*
            }

            fn metadata(&self) -> Vec<TypeMetadata> {
                use std::alloc::Layout;
                vec![
                    $(TypeMetadata {
                        layout: Layout::new::<$type>()
                    },)*
                ]
            }

            fn read_entity(archetype: &'a Archetype, entity: Entity) -> Result<Self::Ref> {
                let data_index = archetype.data_index_for_entity(entity);
                Ok((
                    $(archetype.read_component::<$type>($index, data_index, std::mem::size_of::<$type>()),)*
                ))
            }

            fn read_entity_mut(archetype: &'a Archetype, entity: Entity) -> Result<Self::RefMut> {
                let data_index = archetype.data_index_for_entity(entity);
                Ok((
                    $(archetype.read_component_mut::<$type>($index, data_index, std::mem::size_of::<$type>()),)*
                ))
            }
        }
    }
}

impl_component_bundle!(A => 0,);
impl_component_bundle!(A => 0, B => 1,);
impl_component_bundle!(A => 0, B => 1, C => 2,);
impl_component_bundle!(A => 0, B => 1, C => 2, D => 3,);
impl_component_bundle!(A => 0, B => 1, C => 2, D => 3, E => 4,);
impl_component_bundle!(A => 0, B => 1, C => 2, D => 3, E => 4, F => 5,);

pub struct TypeMetadata {
    layout: std::alloc::Layout,
}

fn align_value(value: usize, alignment: usize) -> usize {
    value + alignment - 1 & !(alignment - 1)
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

    #[test]
    fn align_unaligned_value() {
        assert_eq!(align_value(35, 8), 40);
    }

    #[test]
    fn align_aligned_value() {
        assert_eq!(align_value(8, 8), 8);
        assert_eq!(align_value(16, 8), 16);
    }
}
