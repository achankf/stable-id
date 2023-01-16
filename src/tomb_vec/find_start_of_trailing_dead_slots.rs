use crate::{CastUsize, Slot};

/**
Used as an utility function as part of [`Tec::remove()`], returns
- index which indicates the minimum index to deallocate memory
- number of items to deallocate (using vec.pop())
 */
pub(crate) fn find_start_of_trailing_dead_slots<IndexT: CastUsize, DataT>(
    slice: &[Slot<DataT, IndexT>],
) -> Option<(IndexT, usize)> {
    let len = slice.len();

    if slice.is_empty() {
        return None;
    }

    let len_minus_one = len - 1;
    let mut acc: Option<usize> = None;
    let mut count = 0;

    for (i, slot) in slice.iter().rev().enumerate() {
        let index = len_minus_one - i;

        if matches!(slot, Slot::Alive(_)) {
            return acc.map(|index| (IndexT::from(index), count));
        }

        count += 1;
        acc = Some(index);
    }

    // all items in the slice are dead
    Some((IndexT::from(0), count))
}

#[cfg(test)]
mod tests {
    use crate::{
        tomb_vec::find_start_of_trailing_dead_slots::find_start_of_trailing_dead_slots, Slot,
    };

    #[test]
    fn base_case1() {
        let data: [Slot<usize, usize>; 0] = [];
        let result = find_start_of_trailing_dead_slots(&data);
        assert_eq!(result, None);
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
