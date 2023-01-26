use std::mem;

use stable_id_traits::{Maximum, Predecessor, Successor};

use crate::Eids;

impl<IndexT> Eids<IndexT>
where
    IndexT: Successor + Predecessor + Clone + Copy + Ord + Maximum,
{
    pub fn claim(&mut self) -> IndexT {
        assert!(
            self.next < IndexT::max_value(),
            "storing more items than you can address"
        );

        self.freed
            .iter()
            .next()
            .cloned()
            .map(|id| {
                // found an id in the free list, return it
                let is_removed = self.freed.remove(&id);
                assert!(is_removed, "freeing something not in the database");
                id
            })
            .unwrap_or_else(|| {
                // otherwise increment the id and return it
                let next = self.next.next_value();
                mem::replace(&mut self.next, next)
            })
    }

    pub fn unclaim(&mut self, val: IndexT) {
        assert!(val < self.next, "not a valid entity");

        let is_double_inserted = self.freed.insert(val);
        assert!(is_double_inserted, "double-freeing entity")
    }

    /**
        Pack up recycled ids from the freed list while you deal with the change through `f(old_id, new_id)`.

        ```
        use stable_id::Eids;

        let mut entities: Eids<u8> = Default::default();
        (0..255).for_each(|i| {
            assert_eq!(entities.claim(), i);
        });

        entities.unclaim(27);
        entities.unclaim(15);

        // free up 254 to 251, so that coalescing would be 15 & 27 to takeover 249, 250
        entities.unclaim(254);
        entities.unclaim(252);
        entities.unclaim(251);
        entities.unclaim(253);

        let mut records_old = Vec::new();
        let mut records_new = Vec::new();

        entities.coalesce(|old_id, new_id| {
            records_old.push(old_id);
            records_new.push(new_id);

            // update all data that reference the old_id and replace them with new_id
        });

        assert_eq!(records_old, [250,249]); // reclaiming from the last-issued
        assert_eq!(records_new, [27,15]); // note: larger ids come first
        ```
    */
    pub fn coalesce<F>(&mut self, mut f: F)
    where
        F: FnMut(IndexT, IndexT),
    {
        if self.freed.is_empty() {
            return;
        }

        while let Some(freed) = self.freed.pop_last() {
            let target = self.next.prev_value();
            self.next = target;

            if target != freed {
                f(target, freed);
            }
        }
    }
}

#[cfg(test)]
mod eid_tests {
    use super::Eids;

    #[test]
    fn claim_ids() {
        let mut entities: Eids<u8> = Default::default();
        for i in 0..100 {
            let id = entities.claim();
            assert_eq!(id, i);
        }

        fn is_multiple_of_3(i: &u8) -> bool {
            i % 3 == 0
        }

        (0..60u8)
            .filter(is_multiple_of_3)
            .for_each(|i| entities.unclaim(i));

        (0..60u8)
            .filter(is_multiple_of_3)
            .all(|i| entities.claim() == i);
    }

    #[test]
    #[should_panic]
    fn unclaim_invalid() {
        let mut entities: Eids<u8> = Default::default();
        entities.unclaim(123u8)
    }

    #[test]
    #[should_panic]
    fn double_free() {
        let mut entities: Eids<u8> = Default::default();
        let id = entities.claim();
        entities.unclaim(id);
        entities.unclaim(id);
    }

    #[test]
    #[should_panic]
    fn claim_over_max() {
        let mut entities: Eids<u8> = Default::default();
        (0..257).for_each(|_| {
            entities.claim();
        });
    }
}
