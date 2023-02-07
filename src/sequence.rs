use stable_id_traits::Successor;

use crate::Sequence;

impl<IndexT> Sequence<IndexT>
where
    IndexT: Successor + Clone + Copy,
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
