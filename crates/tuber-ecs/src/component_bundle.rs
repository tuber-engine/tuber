use crate::archetype::{Archetype, TypeMetadata};
use crate::Entity;
use std::any::TypeId;
use tuber_core::Result;

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
                use std::any::TypeId;
                vec![
                    $(TypeMetadata {
                        layout: Layout::new::<$type>(),
                        type_id: TypeId::of::<$type>()
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
