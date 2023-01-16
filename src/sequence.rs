use std::hash::Hash;

use crate::{Sequence, Successor};

impl<IndexT> Sequence<IndexT>
where
    IndexT: Successor + Clone + Copy + Hash + Eq,
{
    pub const fn continue_from(start: IndexT) -> Self {
        Self { counter: start }
    }

    pub fn next(&mut self) -> IndexT {
        let ret = self.counter;
        self.counter = ret.next();
        ret
    }
}
