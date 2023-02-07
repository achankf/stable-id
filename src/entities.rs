use std::{
    hash::Hash,
    ops::{Index, IndexMut},
};

use rustc_hash::FxHashMap;
use stable_id_traits::{CastUsize, Maximum, Successor};

use crate::Tec;

use super::Entities;

impl<DataT, IndexT> Entities<DataT, IndexT>
where
    IndexT: Default + Successor + Clone + Copy + Hash + Eq + CastUsize + Ord + Maximum,
{
    /** Reserves spaces similar to [`Vec::with_capacity()`]. */
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vtable: Default::default(),
            data: Tec::with_capacity(capacity),
            seq: Default::default(),
        }
    }

    /** Returns the number of items in this data structure. */
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /** Tells you if the collection is empty. */
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /** Try getting the item with the given id. */
    pub fn get(&self, index: IndexT) -> Option<&DataT> {
        self.vtable
            .get(&index)
            .and_then(|physical_id| self.data.get(*physical_id).map(|data| data))
    }

    /** Mutable version of get. */
    pub fn get_mut(&mut self, index: IndexT) -> Option<&mut DataT> {
        self.vtable
            .get(&index)
            .and_then(|physical_id| self.data.get_mut(*physical_id).map(|data| data))
    }

    /**
    Removes an element for the given id.
    */
    pub fn remove(&mut self, index: IndexT) -> Option<DataT> {
        let virtual_id = index;
        let physical_id = self.vtable.get(&virtual_id);

        if let Some(&physical_id) = physical_id {
            let data = self.data.remove(physical_id);

            self.vtable.remove(&virtual_id).expect("cannot remove item"); // contradiction: we just found the physical id

            assert_eq!(self.vtable.len(), self.data.len());

            let len = self.len();
            let capacity = self.data.capacity();
            let num_dead_slots = capacity - len;
            let logn = len.checked_ilog2();

            if let Some(logn) = logn {
                // we can perform the cast because log(MAX) is always smaller than MAX
                if num_dead_slots >= logn.cast_to() {
                    self.coalesce();
                }
            } else {
                debug_assert!(len == 0);
            }

            Some(data)
        } else {
            None
        }
    }

    /**
    Allocate an entity with monotonically increase ids, just like [`crate::SparseEntities`].
    */
    pub fn alloc(&mut self, data: DataT) -> IndexT {
        let virtual_id = self.seq.next_value();
        let phyiscal_id = self.data.alloc(data);

        self.vtable.insert(virtual_id, phyiscal_id);

        virtual_id
    }

    /// Return all data's references.
    pub fn iter(&self) -> impl Iterator<Item = &DataT> {
        self.data.iter()
    }

    /// Return all data in mutable references.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DataT> {
        self.data.iter_mut()
    }

    /**
    Iterate every entries. This takes O(`HashMap::iter()`) to iterate the entire collection.
    */
    pub fn iter_with_id(&self) -> impl Iterator<Item = (IndexT, &DataT)> {
        self.vtable.iter().map(|(virtual_id, physical_id)| {
            let data = &self.data[*physical_id];

            (*virtual_id, data)
        })
    }

    /**
    Compact spaces internally.
    */
    fn coalesce(&mut self) {
        let reverse_mapping: FxHashMap<_, _> = self.vtable.iter().map(|(a, b)| (*b, *a)).collect();

        self.data.coalesce(|old_physical_id, new_physical_id| {
            let virtual_id = reverse_mapping
                .get(&old_physical_id)
                .cloned()
                .expect("inconsistent index");

            self.vtable.entry(virtual_id).and_modify(|c| {
                *c = new_physical_id;
            });
        })
    }
}

impl<DataT, IndexT> Default for Entities<DataT, IndexT>
where
    IndexT: Default + Maximum,
{
    fn default() -> Self {
        Self {
            vtable: Default::default(),
            data: Default::default(),
            seq: Default::default(),
        }
    }
}

impl<DataT, IndexT> Index<IndexT> for Entities<DataT, IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq + Default + CastUsize + Ord + Maximum,
{
    type Output = DataT;

    fn index(&self, index: IndexT) -> &Self::Output {
        self.get(index).expect("element not exist")
    }
}

impl<DataT, IndexT> IndexMut<IndexT> for Entities<DataT, IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq + Default + CastUsize + Ord + Maximum,
{
    fn index_mut(&mut self, index: IndexT) -> &mut Self::Output {
        self.get_mut(index).expect("element not exist")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::Entities;

    #[test]
    fn access_out_of_bound() {
        let mut entities = Entities::default();
        entities.alloc(1232);
        assert_eq!(entities.get(312u16), None);
    }

    #[test]
    #[should_panic(expected = "element not exist")]
    fn access_out_of_bound_mut() {
        let mut entities = Entities::default();
        entities.alloc(1232);
        entities[312u16] = 3333;
    }

    #[test]
    fn normal() {
        let mut entities = Entities::default();

        fn check_all(entities: &Entities<&str>) {
            entities
                .iter_with_id()
                .for_each(|(id, data)| assert_eq!(entities[id], *data));
        }

        vec!["0", "1", "2", "3", "4", "5"]
            .into_iter()
            .fold(HashMap::new(), |mut acc, data| {
                acc.insert(entities.alloc(data), data);
                acc
            })
            .into_iter()
            .for_each(|(id, data)| assert_eq!(entities[id], data));

        assert_eq!(entities.remove(1), Some("1"));
        check_all(&entities);

        assert_eq!(entities.remove(4), Some("4"));
        check_all(&entities);

        assert_eq!(entities.remove(5), Some("5"));
        check_all(&entities);

        assert_eq!(entities.remove(3), Some("3"));
        check_all(&entities);

        assert_eq!(entities.remove(2), Some("2"));
        assert_eq!(entities.len(), 1);
        check_all(&entities);

        assert_eq!(entities.remove(0), Some("0"));
        assert!(entities.is_empty());
        check_all(&entities);
    }

    #[test]
    fn iter() {
        let mut entities = Entities::default();

        fn check_all(entities: &Entities<String>) {
            entities
                .iter_with_id()
                .for_each(|(id, data)| assert_eq!(entities[id], *data));
        }

        vec![
            "0".to_owned(),
            "1".to_owned(),
            "2".to_owned(),
            "3".to_owned(),
            "4".to_owned(),
            "5".to_owned(),
        ]
        .into_iter()
        .fold(HashMap::new(), |mut acc, data| {
            acc.insert(entities.alloc(data.clone()), data);
            acc
        })
        .into_iter()
        .for_each(|(id, data)| assert_eq!(entities[id], data));

        assert_eq!(entities.remove(1), Some("1".to_owned()));
        check_all(&entities);

        assert_eq!(entities.remove(4), Some("4".to_owned()));
        check_all(&entities);

        assert_eq!(entities.remove(5), Some("5".to_owned()));
        check_all(&entities);

        assert_eq!(entities.remove(2), Some("2".to_owned()));
        check_all(&entities);

        let data_with_id = HashSet::from([(3, "3".to_owned()), (0, "0".to_owned())]);

        assert_eq!(
            HashSet::from(["3".to_owned(), "0".to_owned()]),
            entities.iter().cloned().collect(),
        );

        assert_eq!(
            data_with_id,
            entities
                .iter_with_id()
                .map(|(id, value)| (id, value.to_owned()))
                .collect(),
        );

        entities
            .iter_mut()
            .for_each(|value| *value = format!("1{value}"));

        assert_eq!(
            HashSet::from([(3, "13".to_owned()), (0, "10".to_owned())]),
            entities
                .iter_with_id()
                .map(|(id, value)| (id, value.to_owned()))
                .collect(),
        );
    }

    #[test]
    fn coalesce_1() {
        let mut entities: Entities<u8, u8> = Default::default();
        (0..255).for_each(|i| {
            assert_eq!(entities.alloc(i), i);
        });

        entities.remove(27);
        entities.remove(254);
        entities.remove(15);
        entities.remove(252);
        entities.remove(251);
        entities.remove(253);

        entities.coalesce();

        let unique_values: HashSet<_> = entities.iter_with_id().map(|(_, data)| *data).collect();

        assert_eq!(unique_values.len(), 249);
    }

    #[test]
    fn coalesce_2() {
        let mut entities: Entities<u8, u8> = Default::default();
        (0..255).for_each(|i| {
            assert_eq!(entities.alloc(i), i);
        });

        entities.remove(27);
        entities.remove(15);

        entities.remove(250);
        entities.remove(232);
        entities.remove(231);
        entities.remove(254);
        entities.remove(252);
        entities.remove(251);
        entities.remove(25);
        entities.remove(253);
        entities.remove(229);
        entities.remove(233);
        entities.remove(234);
        entities.remove(235);
        entities.remove(236);
        entities.remove(237);
        entities.remove(238);
        entities.remove(239);
        entities.remove(240);
        entities.remove(35);
        entities.remove(241);
        entities.remove(242);
        entities.remove(243);
        entities.remove(245);
        entities.remove(244);
        entities.remove(246);
        entities.remove(247);
        entities.remove(248);
        entities.remove(34);
        entities.remove(249);
        entities.remove(30);

        entities.coalesce();

        let unique_values: HashSet<_> = entities.iter_with_id().map(|(_, data)| *data).collect();
        assert_eq!(unique_values.len(), 224);
    }

    #[test]
    fn coalesce_from_remove() {
        let mut entities: Entities<char, u8> = Default::default();

        ['a', 'b', 'c', 'd', 'e'].into_iter().for_each(|c| {
            entities.alloc(c);
        });

        entities.remove(2);
        entities.remove(3);
        entities.remove(1);

        assert_eq!(entities.len(), 2);
        assert_eq!(
            HashSet::from(['a', 'e']),
            entities.iter().cloned().collect()
        );
        assert_eq!(entities.data.capacity(), 2); // coalesce() was called since we removed a majority of items.
    }
}
