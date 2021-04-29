use crate::bitset::BitSet;
use crate::ecs::Components;
use crate::EntityIndex;
use accessors::Accessor;
use std::any::TypeId;
use std::collections::HashSet;
use std::marker::PhantomData;

pub trait Query<'a> {
    type ResultType: 'a;

    fn fetch(index: EntityIndex, components: &'a Components) -> Self::ResultType;
    fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex>;
    fn type_ids() -> Vec<TypeId>;
}

macro_rules! impl_query_tuples {
    ($th:tt, $($t:tt,)*) => {
        impl<'a, $th, $($t,)*> Query<'a> for ($th, $($t,)*)
        where
            $th: Accessor<'a>,
            $($t: Accessor<'a>,)*
        {
            type ResultType = (EntityIndex, ($th::RefType, $($t::RefType,)*));

            fn fetch(index: EntityIndex, components: &'a Components) -> Self::ResultType {
                (index, ($th::fetch(index, components), $($t::fetch(index, components),)*))
            }

            #[allow(unused_mut)]
            fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex> {
                let mut result = $th::matching_ids(entity_count, components);
                $(result = result.intersection(&$t::matching_ids(entity_count, components)).cloned().collect();)*
                result
            }

            fn type_ids() -> Vec<TypeId> {
                vec![$th::type_id(), $($t::type_id(),)*]
            }
        }
    }
}

impl_query_tuples!(A,);
impl_query_tuples!(A, B,);
impl_query_tuples!(A, B, C,);
impl_query_tuples!(A, B, C, D,);
impl_query_tuples!(A, B, C, D, E,);
impl_query_tuples!(A, B, C, D, E, F,);
impl_query_tuples!(A, B, C, D, E, F, G,);
impl_query_tuples!(A, B, C, D, E, F, G, H,);

pub struct QueryIterator<'a, Q> {
    index: EntityIndex,
    components: &'a Components,
    matching_entities: Vec<EntityIndex>,
    marker: PhantomData<&'a Q>,
}

impl<'a, 'b, Q: Query<'b>> QueryIterator<'a, Q> {
    pub fn new(entity_count: usize, components: &'a Components) -> Self {
        let mut bitsets = vec![];
        for type_id in Q::type_ids() {
            if let Some(component_store) = components.get(&type_id) {
                bitsets.push(component_store.entities_bitset.clone());
            }
        }

        let mut matching_entities = vec![];
        if bitsets.len() == Q::type_ids().len() {
            'outer: for i in 0..entity_count {
                for bitset in bitsets.iter() {
                    if !bitset.bit(i) {
                        continue 'outer;
                    }
                }

                matching_entities.push(i);
            }
        }

        Self {
            index: 0,
            components,
            matching_entities,
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
        self.index = if let Some(index) = self.matching_entities.pop() {
            index
        } else {
            return None;
        };
        Some(Q::fetch(self.index, self.components))
    }
}

pub mod accessors {
    use crate::bitset::BitSet;
    use crate::ecs::Components;
    use crate::EntityIndex;
    use std::any::TypeId;
    use std::cell::{Ref, RefMut};
    use std::collections::HashSet;
    use std::marker::PhantomData;

    pub struct R<T>(PhantomData<T>);
    pub struct W<T>(PhantomData<T>);

    pub trait Accessor<'a> {
        type RawType: 'a;
        type RefType: 'a;

        fn fetch(index: usize, components: &'a Components) -> Self::RefType;
        fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex>;
        fn type_id() -> TypeId;
    }
    impl<'a, T: 'static> Accessor<'a> for R<T> {
        type RawType = T;
        type RefType = Ref<'a, T>;

        fn fetch(index: usize, components: &'a Components) -> Self::RefType {
            Ref::map(
                components[&TypeId::of::<T>()].component_data[index]
                    .as_ref()
                    .unwrap()
                    .borrow(),
                |r| r.downcast_ref().unwrap(),
            )
        }

        fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex> {
            let mut result = HashSet::new();
            if let Some(component_store) = components.get(&TypeId::of::<T>()) {
                for i in 0..entity_count.max(component_store.entities_bitset.bit_count()) {
                    if component_store.entities_bitset.bit(i) {
                        result.insert(i);
                    }
                }
            }

            result
        }

        fn type_id() -> TypeId {
            TypeId::of::<T>()
        }
    }
    impl<'a, T: 'static> Accessor<'a> for W<T> {
        type RawType = T;
        type RefType = RefMut<'a, T>;

        fn fetch(index: usize, components: &'a Components) -> Self::RefType {
            RefMut::map(
                components[&TypeId::of::<T>()].component_data[index]
                    .as_ref()
                    .unwrap()
                    .borrow_mut(),
                |r| r.downcast_mut().unwrap(),
            )
        }

        fn matching_ids(entity_count: usize, components: &'a Components) -> HashSet<EntityIndex> {
            let mut result = HashSet::new();
            if let Some(component_store) = components.get(&TypeId::of::<T>()) {
                for i in 0..entity_count.max(component_store.entities_bitset.bit_count()) {
                    if component_store.entities_bitset.bit(i) {
                        result.insert(i);
                    }
                }
            }

            result
        }

        fn type_id() -> TypeId {
            TypeId::of::<T>()
        }
    }
}
