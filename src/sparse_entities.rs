use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Index, IndexMut},
};

use stable_id_traits::{Successor, Zero};

use super::SparseEntities;

impl<DataT, IndexT> SparseEntities<DataT, IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq + Zero,
{
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
            seq: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    pub fn get(&self, index: IndexT) -> Option<&DataT> {
        self.data.get(&index)
    }

    pub fn get_mut(&mut self, index: IndexT) -> Option<&mut DataT> {
        self.data.get_mut(&index)
    }

    /** Panic if index is invalid */
    pub fn remove(&mut self, index: IndexT) -> DataT {
        self.data.remove(&index).expect("id is not value")
    }

    pub fn alloc(&mut self, data: DataT) -> IndexT {
        let next_id = self.seq.next_value();
        self.data.insert(next_id, data);

        next_id
    }

    pub fn iter(&self) -> impl Iterator<Item = (IndexT, &DataT)> {
        self.data
            .iter()
            .map(|(virtual_id, data)| (*virtual_id, data))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (IndexT, &mut DataT)> {
        self.data
            .iter_mut()
            .map(|(virtual_id, data)| (*virtual_id, data))
    }
}

impl<DataT, IndexT> IntoIterator for SparseEntities<DataT, IndexT>
where
    IndexT: Successor + Clone + Copy + Default + Hash + Eq,
{
    type Item = (IndexT, DataT);

    type IntoIter = std::collections::hash_map::IntoIter<IndexT, DataT>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<DataT, IndexT> Default for SparseEntities<DataT, IndexT>
where
    IndexT: Zero,
{
    fn default() -> Self {
        Self {
            data: Default::default(),
            seq: Default::default(),
        }
    }
}

impl<DataT, IndexT> Index<IndexT> for SparseEntities<DataT, IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq + Zero,
{
    type Output = DataT;

    fn index(&self, index: IndexT) -> &Self::Output {
        self.get(index).expect("element not exist")
    }
}

impl<DataT, IndexT> IndexMut<IndexT> for SparseEntities<DataT, IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq + Zero,
{
    fn index_mut(&mut self, index: IndexT) -> &mut Self::Output {
        self.get_mut(index).expect("element not exist")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::SparseEntities;

    #[test]
    fn access_out_of_bound() {
        let mut entities = SparseEntities::default();
        entities.alloc(1232);
        assert_eq!(entities.get(312u16), None);
    }

    #[test]
    #[should_panic(expected = "element not exist")]
    fn access_out_of_bound_mut() {
        let mut entities = SparseEntities::default();
        entities.alloc(1232);
        entities[312u16] = 3333;
    }

    #[test]
    fn normal() {
        let mut entities = SparseEntities::default();

        fn check_all(entities: &SparseEntities<&str>) {
            entities
                .iter()
                .for_each(|(id, data)| assert_eq!(entities[id], *data));
        }

        vec!["1", "2", "3", "4", "5"]
            .into_iter()
            .fold(HashMap::new(), |mut acc, data| {
                acc.insert(entities.alloc(data), data);
                acc
            })
            .into_iter()
            .for_each(|(id, data)| assert_eq!(entities[id], data));

        entities.remove(1);
        check_all(&entities);

        entities.remove(4);
        check_all(&entities);

        entities.remove(3);
        check_all(&entities);

        entities.remove(2);
        assert_eq!(entities.len(), 1);
        check_all(&entities);

        entities.remove(0);
        assert!(entities.is_empty());
        check_all(&entities);
    }
}
