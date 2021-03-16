use crate::Entity;
use std::any::TypeId;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ptr::NonNull;

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

    pub fn match_component<C: 'static>(&self) -> bool {
        self.types_metadata
            .iter()
            .any(|m| m.type_id == TypeId::of::<C>())
    }

    pub fn fetch<C: 'static>(&self) -> ComponentIterator<C> {
        let type_index = self
            .types_metadata
            .iter()
            .position(|m| m.type_id == TypeId::of::<C>())
            .unwrap();
        let component_ptr = unsafe { self.data.as_ptr().add(self.type_offsets[type_index]) };
        ComponentIterator {
            ptr: component_ptr,
            index: 0,
            entity_count: self.entity_count,
            phantom: PhantomData,
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

pub struct ComponentIterator<'a, C> {
    ptr: *const u8,
    index: usize,
    entity_count: usize,
    phantom: PhantomData<&'a C>,
}

impl<'a, C> Iterator for ComponentIterator<'a, C> {
    type Item = &'a C;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entity_count {
            return None;
        }

        let component =
            unsafe { (self.ptr.add(self.index * std::mem::size_of::<C>()) as *const C).as_ref() };
        self.index += 1;
        component
    }
}

pub struct TypeMetadata {
    pub(crate) layout: std::alloc::Layout,
    pub(crate) type_id: TypeId,
}

fn align_value(value: usize, alignment: usize) -> usize {
    value + alignment - 1 & !(alignment - 1)
}

#[cfg(test)]
mod tests {
    use super::*;

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
