use std::hash::Hash;

use stable_id_traits::Successor;

use crate::Sequence;

impl<IndexT> Sequence<IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq,
{
    pub const fn continue_from(start: IndexT) -> Self {
        Self { counter: start }
    }

    pub fn next_value(&mut self) -> IndexT {
        let ret = self.counter;
        self.counter = ret.next_value();
        ret
    }
}

impl<IndexT> Default for Sequence<IndexT>
where
    IndexT: stable_id_traits::Zero,
{
    fn default() -> Self {
        Self {
            counter: IndexT::zero(),
        }
    }
}
