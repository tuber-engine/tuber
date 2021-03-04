use std::any::TypeId;
use std::collections::HashMap;
use std::ptr::NonNull;
use tuber_core::Result;

pub type Entity = usize;

pub struct Ecs {
    archetype_store: HashMap<Box<[TypeId]>, Archetype>,
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

    pub fn insert<CB: ComponentBundle>(&mut self, component_bundle: CB) -> Result<Entity> {
        let entity = self.next_entity;
        let archetype = self
            .archetype_store
            .entry(component_bundle.type_ids())
            .or_insert(Archetype::new(component_bundle.metadata()));
        let data_index = archetype.allocate_storage_for_entity(entity);
        component_bundle.write_into(archetype, data_index);
        self.next_entity += 1;
        Ok(entity)
    }

    pub fn entity<CB: ComponentBundle>(&self, entity: Entity) {}
}

pub struct Archetype {
    data: NonNull<u8>,
    capacity: usize,
    entity_count: usize,
    size: usize,
    types_metadata: Vec<TypeMetadata>,
    type_offsets: Vec<usize>,
    stored_entities: Vec<Entity>,
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
            stored_entities: vec![],
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

        self.stored_entities.push(entity);
        self.entity_count += 1;
        self.entity_count - 1
    }

    fn grow(&mut self, new_capacity: usize) {
        use std::alloc::{alloc, dealloc, Layout};

        let computed_size = self.compute_required_size_for_capacity(new_capacity);
        dbg!(computed_size);
        let mut new_data = NonNull::dangling();

        let alignment = self
            .types_metadata
            .first()
            .map_or(1, |tm| tm.layout.align());
        unsafe {
            new_data = NonNull::new(alloc(
                Layout::from_size_align(computed_size, alignment).unwrap(),
            ))
            .unwrap();

            if self.capacity != 0 {
                dealloc(
                    self.data.as_ptr(),
                    Layout::from_size_align(self.size, alignment).unwrap(),
                )
            }
        }

        self.data = new_data;
        self.capacity = new_capacity;
        self.size = computed_size;
    }

    fn compute_required_size_for_capacity(&mut self, capacity: usize) -> usize {
        let mut size = 0;
        for (i, type_metadata) in self.types_metadata.iter().enumerate() {
            size = align_value(size, type_metadata.layout.align());
            self.type_offsets[i] = size;
            size += type_metadata.layout.size() * capacity;
        }
        size
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

    fn component_ptr(&self, data_index: usize, data_size: usize, type_index: usize) -> *mut u8 {
        let type_offset = self.type_offsets[type_index];

        unsafe {
            self.data
                .as_ptr()
                .add(type_offset + data_index * data_size)
                .cast::<u8>()
        }
    }
}

pub trait ComponentBundle {
    fn type_ids(&self) -> Box<[TypeId]>;
    fn write_into(&self, archetype: &mut Archetype, entity: Entity);
    fn metadata(&self) -> Vec<TypeMetadata>;
}

impl<A: 'static, B: 'static> ComponentBundle for (A, B) {
    fn type_ids(&self) -> Box<[TypeId]> {
        Box::new([TypeId::of::<A>(), TypeId::of::<B>()])
    }

    fn write_into(&self, archetype: &mut Archetype, data_index: usize) {
        archetype.write_component(
            0,
            data_index,
            std::mem::size_of::<A>(),
            &self.0 as *const A as *const u8,
        );
        archetype.write_component(
            1,
            data_index,
            std::mem::size_of::<B>(),
            &self.1 as *const B as *const u8,
        );
    }

    fn metadata(&self) -> Vec<TypeMetadata> {
        use std::alloc::Layout;
        vec![
            TypeMetadata {
                layout: Layout::new::<A>(),
            },
            TypeMetadata {
                layout: Layout::new::<B>(),
            },
        ]
    }
}

pub struct TypeMetadata {
    layout: std::alloc::Layout,
}

fn align_value(value: usize, alignment: usize) -> usize {
    value + alignment - 1 & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Position {
        pub x: f32,
        pub y: f32,
    }

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
        ecs.insert((Position { x: 2.0, y: 1.0 }, Velocity { x: 1.5, y: 2.6 }))
            .unwrap();
        assert_eq!(ecs.entity_count(), 1usize);
    }

    #[test]
    fn ecs_insert_two_entities() {
        let mut ecs = Ecs::new();
        ecs.insert((Position { x: 2.0, y: 1.0 }, Velocity { x: 1.5, y: 2.6 }))
            .unwrap();
        ecs.insert((Position { x: 0.2, y: 0.5 }, Velocity { x: 3.0, y: 2.0 }))
            .unwrap();
        assert_eq!(ecs.entity_count(), 2usize);
    }

    #[test]
    fn ecs_entity() {
        let mut ecs = Ecs::new();
        ecs.insert((Position { x: 2.0, y: 1.0 }, Velocity { x: 1.5, y: 2.6 }))
            .unwrap();
        /*let (position, velocity) = ecs.entity::<(Position, Velocity)>(0);
        assert_eq!(position, Position { x: 2.0, y: 1.0 });
        assert_eq!(velocity, Velocity { x: 1.5, y: 2.6 });*/
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
