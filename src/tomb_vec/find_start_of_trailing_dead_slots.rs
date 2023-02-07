use std::ops::ControlFlow;

use stable_id_traits::CastUsize;

use crate::Slot;

/**
Used as an utility function as part of [`Tec::remove()`], returns
- index which indicates the minimum index to deallocate memory
- number of items to deallocate (using vec.pop())
 */
pub(crate) fn find_start_of_trailing_dead_slots<IndexT: CastUsize, DataT>(
    slice: &[Slot<DataT, IndexT>],
) -> Option<(IndexT, usize)> {
    // This is a helper function of another helper function `remove_trailing_dead_slots` for `remove()`.
    // Getting to this point means one living slot is turned into a dead slot -- i.e. slice shouldn't be empty.
    debug_assert!(!slice.is_empty());

    let result = slice.iter().rev().try_fold((false, 0usize), |acc, slot| {
        if matches!(slot, Slot::Alive(_)) {
            ControlFlow::Break(acc)
        } else {
            let (_, count) = acc;
            ControlFlow::Continue((true, count + 1))
        }
    });

    let (has_removable_candidates, count) = match result {
        ControlFlow::Break(count) => count,
        ControlFlow::Continue(count) => count,
    };

    if has_removable_candidates {
        Some((IndexT::cast_from(slice.len() - count), count))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        tomb_vec::find_start_of_trailing_dead_slots::find_start_of_trailing_dead_slots, Slot,
    };

    #[test]
    #[should_panic]
    fn base_case1() {
        let data: [Slot<usize, usize>; 0] = [];
        find_start_of_trailing_dead_slots(&data);
    }

    #[test]
    fn base_case2() {
        let data: [Slot<usize, usize>; 1] = [Slot::Alive(1232)];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, None);
    }

    #[test]
    fn test1() {
        let data: Vec<Slot<usize, usize>> = vec![
            Slot::Alive(324),
            Slot::Dead { next_free: 1 },
            Slot::Alive(34),
            Slot::Dead { next_free: 2 },
            Slot::Dead { next_free: 3 },
            Slot::Dead { next_free: 4 },
            Slot::Dead { next_free: 5 },
            Slot::Dead { next_free: 6 },
            Slot::Dead { next_free: 7 },
        ];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, Some((3, 6)));
    }

    #[test]
    fn test2() {
        let data: Vec<Slot<usize, usize>> = vec![
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
        ];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, Some((0, 7)));
    }

    #[test]
    fn test3() {
        let data: Vec<Slot<usize, usize>> = vec![
            Slot::Alive(324),
            Slot::Dead { next_free: 1 },
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
        ];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, Some((7, 6)));
    }

    #[test]
    fn test4() {
        let data: Vec<Slot<usize, usize>> = vec![
            Slot::Alive(324),
            Slot::Dead { next_free: 1 },
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Alive(34),
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
        ];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, Some((11, 3)));
    }

    #[test]
    fn test5() {
        let data: Vec<Slot<usize, usize>> = vec![
            Slot::Alive(324),
            Slot::Dead { next_free: 1 },
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Alive(34),
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Dead { next_free: 1 },
            Slot::Alive(34),
        ];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, None);
    }
}
