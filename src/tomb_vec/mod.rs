mod tomb_vec_tests;

use std::fmt::Debug;

use std::cmp::Reverse;
use std::collections::BinaryHeap;

use std::{
    mem,
    ops::{Index, IndexMut},
};

use stable_id_traits::{CastUsize, Maximum};

use crate::{Slot, Tec};

impl<IndexT, DataT> Default for Tec<IndexT, DataT>
where
    IndexT: Maximum,
{
    fn default() -> Self {
        Self {
            vec: Default::default(),
            next_free: Maximum::max_value(),
            count: 0,
        }
    }
}

impl<IndexT, DataT> Tec<IndexT, DataT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
{
    fn set_sentinal(&mut self) {
        self.next_free = Maximum::max_value();
    }

    fn check_free_link_invariant(&self, link: IndexT) -> bool {
        let n = link.cast_to();
        let m = IndexT::max_value().cast_to();

        // either the free list link is pointing to a valid spot in memory
        // or it's pointing to the sentinal
        n <= self.capacity() || n == m
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            vec: Vec::with_capacity(capacity),
            ..Self::default()
        }
    }

    /// Number of items in this data structure.
    pub fn len(&self) -> usize {
        debug_assert_eq!(
            self.iter().count(),
            self.count,
            "number of living items doesn't match self.count"
        );
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.vec.clear();
        self.count = 0;
        self.set_sentinal();
    }

    /**
    Allocates an id from the given `data`.
    Note: can store at most IndexT::max_value() - 1 elements, because
    the next free node needs to be count + 1.
    */
    pub fn alloc(&mut self, data: DataT) -> IndexT {
        let original_free_index = self.next_free;

        let next_slot = self.vec.get_mut(original_free_index.cast_to());

        let result_index = if let Some(slot) = next_slot {
            match slot {
                Slot::Alive(..) => unimplemented!("next free slot is already occupied"),
                Slot::Dead { next_free } => {
                    self.next_free = *next_free;
                    *slot = Slot::Alive(data);
                }
            }
            original_free_index
        } else {
            let result_index = self.capacity();

            assert!(
                result_index < IndexT::max_value().cast_to(),
                "exceed storage limit"
            );

            self.vec.push(Slot::Alive(data));
            self.set_sentinal();
            IndexT::cast_from(result_index)
        };

        self.count += 1;

        debug_assert!(self.check_consistency());

        result_index
    }

    /** Panic if index is invalid */
    pub fn remove(&mut self, index: IndexT) -> DataT {
        assert!(!self.is_empty(), "removing an item from an empty container");

        // invariants: the free index must be either
        //      - pointer some dead slot within the vec
        //      - or the end of the vector

        // we're doing panic! over Option, so just do the bookkeeping here since we don't need to recover anything
        self.count -= 1;

        let index_usize = index.cast_to();
        let removal_candidate = &mut self.vec[index_usize];

        let data = match removal_candidate {
            Slot::Alive(_) => {
                // create a dead slot and then swap it with the candidate
                let mut temp_dead_slot = Slot::Dead {
                    next_free: self.next_free,
                };
                mem::swap(&mut temp_dead_slot, removal_candidate);

                // the temporary slot now has the removed item

                self.next_free = index;

                match temp_dead_slot {
                    Slot::Alive(data) => data,
                    Slot::Dead { .. } => unreachable!("cannot unwrap a dead item"),
                }
            }
            Slot::Dead { .. } => panic!("removing a dead item"),
        };

        data
    }

    pub fn get(&self, index: IndexT) -> Option<&DataT> {
        self.vec.get(index.cast_to()).and_then(|slot| match slot {
            Slot::Alive(data) => Some(data),
            Slot::Dead { .. } => None,
        })
    }

    pub fn get_mut(&mut self, index: IndexT) -> Option<&mut DataT> {
        self.vec
            .get_mut(index.cast_to())
            .and_then(|slot| match slot {
                Slot::Alive(data) => Some(data),
                Slot::Dead { .. } => None,
            })
    }

    pub fn iter(&self) -> impl Iterator<Item = &DataT> + DoubleEndedIterator {
        self.vec.iter().filter_map(|data| match data {
            Slot::Alive(data) => Some(data),
            Slot::Dead { .. } => None,
        })
    }

    pub fn iter_with_id(&self) -> impl Iterator<Item = (IndexT, &DataT)> + DoubleEndedIterator {
        self.vec
            .iter()
            .enumerate()
            .filter_map(|(id, data)| match data {
                Slot::Alive(data) => Some((IndexT::cast_from(id), data)),
                Slot::Dead { .. } => None,
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut DataT> + DoubleEndedIterator {
        self.vec.iter_mut().filter_map(|data| match data {
            Slot::Alive(data) => Some(data),
            Slot::Dead { .. } => None,
        })
    }

    pub fn iter_mut_with_id(
        &mut self,
    ) -> impl Iterator<Item = (IndexT, &mut DataT)> + DoubleEndedIterator {
        self.vec
            .iter_mut()
            .enumerate()
            .filter_map(|(id, data)| match data {
                Slot::Alive(data) => Some((CastUsize::cast_from(id), data)),
                Slot::Dead { .. } => None,
            })
    }

    pub fn into_iter_with_id(self) -> impl Iterator<Item = (IndexT, DataT)> + DoubleEndedIterator {
        self.vec
            .into_iter()
            .enumerate()
            .filter_map(|(id, data)| match data {
                Slot::Alive(data) => Some((CastUsize::cast_from(id), data)),
                Slot::Dead { .. } => None,
            })
    }

    /// The amount of occupied space in the underlying `vec`.
    /// Note:
    /// ```compile_fail
    /// self.len() <= self.capacity() == self.vec.len() <= self.vec.capacity()
    /// ```
    pub fn capacity(&self) -> usize {
        self.vec.len()
    }

    fn get_free_list(&self) -> Vec<IndexT> {
        let max = Maximum::max_value();
        let capacity = self.capacity();
        let len = self.len();
        assert!(capacity >= len);

        let mut cur = self.next_free;
        let mut acc = Vec::with_capacity(capacity - len);

        loop {
            if cur == max {
                break;
            }

            if let Slot::Dead { next_free } = &self.vec[cur.cast_to()] {
                acc.push(cur);
                cur = *next_free;
            } else {
                unreachable!("found a living slot in free list")
            }
        }
        acc
    }

    /**
    Coalescing using the typical typical 2 direction trick, and then return the number of items being removed.
    - FORWARD: we need to backfill dead slots in increasing order, using a binary heap
    - another cursor traverse from the back of the memory block to scan for living slots and do the swap

    However, if you can bound the number of dead slots to k=log(n), then you can bound this to O(log n). Analysis:
    - forward cusor: it uses a binary heap to iterate, which takes `k log k` = `log(n) * log(log(n))` comparisons, which is O(log(n)), calculation thanks to symbolic calculator.
    - backward cursor: either it gets k living members, or it has loop through at most k dead members to get the k living memebers, so O(k) = O(log(n))
    */
    fn heap_based_coalesce<F>(&mut self, mut f: F) -> usize
    where
        F: FnMut(IndexT, IndexT),
    {
        let mut free_heap = {
            let free_list: Vec<_> = self.get_free_list().into_iter().map(Reverse).collect();

            BinaryHeap::from(free_list)
        };
        let removed_len = free_heap.len();

        let mut backward_cursor = self.capacity() - 1;
        let max = Maximum::max_value();
        'main_loop: while let Some(Reverse(forward_cursor)) = free_heap.pop() {
            // find a living slot from the back
            let mut living_target = loop {
                let swap_target = &mut self.vec[backward_cursor];

                let forward_cursor_usize = forward_cursor.cast_to();
                if forward_cursor_usize >= backward_cursor {
                    break 'main_loop;
                }

                if matches!(swap_target, Slot::Alive(_)) {
                    // Let's swap the target out of the vec and replace with garbage data.
                    // Later self.remove_trailing_dead_slots() will drop them.
                    let mut dummy = Slot::Dead { next_free: max };
                    mem::swap(swap_target, &mut dummy);
                    break dummy;
                }

                backward_cursor -= 1;

                // note: we have at least 1 living slot otherwise the code would short circuit in the base case
                debug_assert!(backward_cursor != 0);
            };

            let dead_target = &mut self.vec[forward_cursor.cast_to()];
            debug_assert!(matches!(dead_target, Slot::Dead { .. }));

            // i.e. doing a remove and swap
            mem::swap(&mut living_target, dead_target);
            f(IndexT::cast_from(backward_cursor), forward_cursor);
        }

        removed_len
    }

    /**
    Coalesce the data by removing the dead slots. Takes a function `f(old_id, new_id)`
    that allows you to deal with changes made by the process, i.e. say in your game model,
    you have an entity which occupied `old_id`, you would need to change all references
    to use the `new_id`.
    This is intended to be used before saving a game.

    Note: this algorithm is O(n lg n) due to the use of binary heap.
    */
    pub fn coalesce<F>(&mut self, f: F)
    where
        F: FnMut(IndexT, IndexT),
    {
        let next_usize = self.next_free.cast_to();
        let capacity = self.capacity();
        if next_usize >= capacity {
            return;
        } else {
            // this implies there is at least 1 living item
            debug_assert!(!self.is_empty());
        }

        let removed_len = self.heap_based_coalesce(f);

        // pop out all trailing dead slots
        self.vec.truncate(capacity - removed_len);

        // edge-case: at this point the memory is compact, so we're pointing the free-list to the sentinel value
        self.set_sentinal();

        debug_assert_eq!(self.len(), self.capacity());
    }

    fn check_consistency(&self) -> bool {
        use std::collections::HashSet;

        debug_assert!(self.check_free_link_invariant(self.next_free));

        if self.is_empty() {
            debug_assert!(self.next_free == IndexT::max_value());
            debug_assert!(self.vec.is_empty());
            return true;
        }

        // indices of all dead slots
        let dead_set: HashSet<usize> = self
            .vec
            .iter()
            .enumerate()
            .filter(|(_, slot)| matches!(slot, Slot::Dead { .. }))
            .map(|(i, _)| i)
            .collect();

        let linked_dead_set = self
            .get_free_list()
            .into_iter()
            .map(CastUsize::cast_to)
            .collect();

        // we're double-counting:
        // - dead_set is based on linear scan of the whole memory
        // - linked_dead_set is based on linked-list traversal from self.next_free
        assert_eq!(dead_set, linked_dead_set);

        true
    }
}

impl<IndexT, DataT> Tec<IndexT, DataT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
    DataT: Clone,
{
    /**
    Populate `count` number of items by cloning the given `data`.
    */
    pub fn populate(data: DataT, count: usize) -> Self {
        let vec = vec![Slot::Alive(data); count];
        let count = vec.len();

        Self {
            vec,
            next_free: Maximum::max_value(),
            count,
        }
    }
}

impl<IndexT, DataT> Tec<IndexT, DataT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
    DataT: Clone + Default,
{
    /**
    Populate `count` number of items with the default value.
    */
    pub fn populate_defaults(count: usize) -> Self {
        Self::populate(Default::default(), count)
    }
}

impl<IndexT, DataT> Tec<IndexT, DataT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
    DataT: Default,
{
    pub fn alloc_default(&mut self) -> IndexT {
        self.alloc(Default::default())
    }
}

impl<IndexT, DataT> Index<IndexT> for Tec<IndexT, DataT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
{
    type Output = DataT;

    fn index(&self, index: IndexT) -> &Self::Output {
        self.get(index).expect("element not exist")
    }
}

impl<IndexT, DataT> IndexMut<IndexT> for Tec<IndexT, DataT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
{
    fn index_mut(&mut self, index: IndexT) -> &mut Self::Output {
        self.get_mut(index).expect("element not exist")
    }
}

impl<IndexT, DataT> Debug for Tec<IndexT, DataT>
where
    IndexT: Debug,
    DataT: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tec")
            .field("vec", &self.vec)
            .field("next_free", &self.next_free)
            .field("count", &self.count)
            .finish()
    }
}
