mod find_start_of_trailing_dead_slots;
mod tomb_vec_tests;

use std::fmt::Debug;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::{
    mem,
    ops::{Index, IndexMut},
};

use stable_id_traits::{CastUsize, Maximum};

use crate::tomb_vec::find_start_of_trailing_dead_slots::find_start_of_trailing_dead_slots;
use crate::{Slot, Tec};

impl<DataT, IndexT> Default for Tec<DataT, IndexT>
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

impl<DataT, IndexT> Tec<DataT, IndexT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
{
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
        self.next_free = IndexT::max_value();
        self.count = 0;
    }

    /**
    Allocates an id from the given `data`.
    Note: can store at most IndexT::max_value() - 1 elements, because
    the next free node needs to be count + 1.
    */
    pub fn alloc(&mut self, data: DataT) -> IndexT {
        let len = self.len();

        assert!(len < IndexT::max_value().cast_to(), "exceed storage limit");

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
            self.vec.push(Slot::Alive(data));
            self.next_free = Maximum::max_value();
            IndexT::cast_from(result_index)
        };

        self.count += 1;

        debug_assert!(self.check_consistency());

        result_index
    }

    /**
    After removing an item, the target slot might expose many dead slots in the end of the vec.
    We need to remove them to maintain the invariant that trailing slot must be alive.
     */
    fn remove_trailing_dead_slots(&mut self) {
        let result = find_start_of_trailing_dead_slots(&self.vec);

        if let Some((last_alive_length, remove_count)) = result {
            let capacity = self.capacity();

            if remove_count == capacity {
                self.clear();
                return;
            }

            // do a linked-list-style "retain()" to remove anything at and beyond `last_trailing_dead_slot`

            // 2 cursors for traversing the linked list:
            // - one for linear scan
            // - and another one for removing/skipping trailing dead slots
            let mut cursor = self.next_free;

            if cursor.cast_to() == capacity {
                return;
            }

            let mut retained_slot_cursor: Option<IndexT> = None;

            // tail of linked-list is while an index points to `len` (just one outside the `vec`)
            loop {
                // check the next item in the link
                if let Slot::Dead { next_free } = self.vec[cursor.cast_to()] {
                    if next_free.cast_to() >= capacity {
                        break;
                    }

                    cursor = next_free;

                    if next_free < last_alive_length {
                        if let Some(prev_keep) = retained_slot_cursor {
                            // remove trailing dead slots in between by updating the link between 2 dead slots
                            if let Slot::Dead { next_free } = &mut self.vec[prev_keep.cast_to()] {
                                *next_free = cursor;
                            } else {
                                unreachable!("reaching an Alive slot when traversing the free list")
                            }
                        } else if next_free < last_alive_length {
                            // update the head link that's stored in the Tec struct
                            self.next_free = next_free;
                        }

                        retained_slot_cursor = Some(cursor);
                    }
                } else {
                    unreachable!("found an alive slot in the free list");
                }
            }

            // deallocate trailing dead slots
            for _ in 0..remove_count {
                let removed = self
                    .vec
                    .pop()
                    .expect("should be able to remove trailing dead slot");
                assert!(matches!(removed, Slot::Dead { .. }));
            }

            // updating the tail to the max
            if self.capacity() == 0 {
                self.clear(); // this updates metadata after popping the vec
            } else if let Some(prev_keep) = retained_slot_cursor {
                if let Slot::Dead { next_free } = &mut self.vec[prev_keep.cast_to()] {
                    *next_free = Maximum::max_value();
                }
            }
        }

        debug_assert!(self.check_consistency());
    }

    /** Panic if index is invalid */
    pub fn remove(&mut self, index: IndexT) -> DataT {
        assert!(
            index < Maximum::max_value(),
            "trying to remove an out of bound item"
        );

        let capacity = self.capacity();
        assert!(capacity > 0, "removing an item from an empty container");

        // invariants: the free index must be either
        //      - pointer some dead slot within the vec
        //      - or the end of the vector

        // we're doing panic! over Option, so just do the bookkeeping here since we don't need to recover anything
        self.count -= 1;

        let removal_candidate = &mut self.vec[index.cast_to()];

        let data = match removal_candidate {
            Slot::Alive(_) => {
                // create a dead slot and then swap it with the candidate
                let mut temp_dead_slot = Slot::Dead {
                    next_free: self.next_free,
                };
                mem::swap(&mut temp_dead_slot, removal_candidate);

                // the temporary slot now has the removed item

                self.next_free = index; // make the removal target as the head of free list

                match temp_dead_slot {
                    Slot::Alive(data) => data,
                    Slot::Dead { .. } => panic!("cannot unwrap a dead item"),
                }
            }
            Slot::Dead { .. } => panic!("removing a dead item"),
        };

        self.remove_trailing_dead_slots();

        debug_assert!(self.check_consistency());

        data
    }

    pub fn get(&self, index: IndexT) -> Option<&DataT> {
        assert!(index < Maximum::max_value(), "index is out of bound");
        self.vec.get(index.cast_to()).and_then(|slot| match slot {
            Slot::Alive(data) => Some(data),
            Slot::Dead { .. } => None,
        })
    }

    pub fn get_mut(&mut self, index: IndexT) -> Option<&mut DataT> {
        assert!(index < Maximum::max_value(), "index is out of bound");
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

    /// The ratio of how much living data vs all data. Use this to determine when to coalesce the data.
    pub fn utility_ratio(&self) -> f64 {
        let capacity = self.capacity();
        if capacity == 0 {
            // assume empty to be fully utilized
            1.
        } else {
            let live = self.len();
            (live as f64) / (capacity as f64)
        }
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
    Coalesce the data by removing the dead slots. Takes a function "f"
    that allows you to deal with changes made by the process.

    Note: this algorithm is O(n lg n) due to the use of binary heap.
    */
    pub fn coalesce<F>(&mut self, mut f: F)
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

        // typical 2 direction trick:
        // - FORWARD: we need to backfill dead slots in increasing order, using a binary heap
        // - another cursor traverse from the back of the memory block to scan for living slots and do the swap

        let mut free_heap = {
            let free_list: Vec<_> = self
                .get_free_list()
                .into_iter()
                .map(|index| Reverse(index))
                .collect();

            BinaryHeap::from(free_list)
        };

        let mut back_cursor = capacity - 1;
        let max = Maximum::max_value();
        'main_loop: while let Some(cursor) = free_heap.pop() {
            let Reverse(cursor) = cursor;

            // find a living slot from the back
            let mut living_target = loop {
                let swap_target = &mut self.vec[back_cursor];

                if cursor.cast_to() >= back_cursor {
                    break 'main_loop;
                }

                if matches!(swap_target, Slot::Alive(_)) {
                    // Let's swap the target out of the vec and replace with garbage data.
                    // Later self.remove_trailing_dead_slots() will drop them.
                    let mut dummy = Slot::Dead { next_free: max };
                    mem::swap(swap_target, &mut dummy);
                    break dummy;
                }

                back_cursor -= 1;

                debug_assert!(back_cursor != 0); // note: we have at least 1 living slot otherwise the code would short circuit in the base case
            };

            let dead_target = &mut self.vec[cursor.cast_to()];
            debug_assert!(matches!(dead_target, Slot::Dead { .. }));

            mem::swap(&mut living_target, dead_target);
            f(cursor, IndexT::cast_from(back_cursor));
        }

        // pop out all trailing dead slots
        let mut back_cursor = capacity - 1;
        loop {
            match self.vec[back_cursor] {
                Slot::Alive(_) => {
                    break;
                }
                Slot::Dead { .. } => {
                    self.vec.pop();
                    back_cursor -= 1;
                    debug_assert!(back_cursor != 0); // note: we have at least 1 living slot otherwise the code would short circuit in the base case
                }
            }
        }
    }

    fn check_consistency(&self) -> bool {
        use std::collections::HashSet;

        if self.is_empty() {
            assert!(self.next_free == IndexT::max_value());
            assert!(self.vec.is_empty());
            return true;
        }

        let dead_set: HashSet<usize> = self
            .vec
            .iter()
            .enumerate()
            .filter(|(_, slot)| matches!(slot, Slot::Dead { .. }))
            .map(|(i, _)| i)
            .collect();

        let mut linked_dead_set = HashSet::with_capacity(dead_set.len());
        let mut cursor = self.next_free;
        let capacity = self.capacity();
        let max = Maximum::max_value();
        linked_dead_set.insert(cursor.cast_to());

        loop {
            let cursor_usize = cursor.cast_to();
            if cursor == max {
                break;
            } else if cursor_usize >= capacity {
                unreachable!("cursor is out of range");
            }

            if let Slot::Dead { next_free } = &self.vec[cursor.cast_to()] {
                cursor = *next_free;
                linked_dead_set.insert(cursor.cast_to());
            }
        }

        // the set from traversed linked list has an extra termination cursor to the capacity (next insertion candidate)
        // invariant: dead_set = linked_dead_set - { max }
        {
            assert_eq!(dead_set.difference(&linked_dead_set).count(), 0);
            let mut diff = linked_dead_set.difference(&dead_set);
            assert_eq!(diff.next().cloned(), Some(max.cast_to()));
            assert_eq!(diff.next().cloned(), None);
        }

        true
    }
}

impl<DataT, IndexT> Tec<DataT, IndexT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
    DataT: Clone,
{
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

impl<DataT, IndexT> Tec<DataT, IndexT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
    DataT: Clone + Default,
{
    pub fn populate_defaults(count: usize) -> Self {
        Self::populate(Default::default(), count)
    }
}

impl<DataT, IndexT> Index<IndexT> for Tec<DataT, IndexT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
{
    type Output = DataT;

    fn index(&self, index: IndexT) -> &Self::Output {
        self.get(index).expect("element not exist")
    }
}

impl<DataT, IndexT> IndexMut<IndexT> for Tec<DataT, IndexT>
where
    IndexT: CastUsize + Ord + Copy + Maximum,
{
    fn index_mut(&mut self, index: IndexT) -> &mut Self::Output {
        self.get_mut(index).expect("element not exist")
    }
}

impl<DataT, IndexT> Debug for Tec<DataT, IndexT>
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
