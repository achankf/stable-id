#[cfg(test)]
mod tests {

    use std::collections::{HashMap, HashSet};

    use stable_id_traits::CastUsize;

    use crate::Tec;

    #[derive(
        Default,
        Clone,
        Copy,
        Ord,
        PartialOrd,
        Eq,
        PartialEq,
        derive_stable_id::StableId,
        Debug,
        Hash,
    )]
    struct Id8(u8);

    #[test]
    fn populate() {
        let count = 50;
        let mut entities = Tec::<usize, u8>::populate_defaults(count);

        assert_eq!(entities.len(), count);
        assert_eq!(entities.alloc(54354534), count as u8);
        assert_eq!(entities.len(), count + 1);
    }

    #[test]
    fn create_remove_end_custom_id() {
        let mut entities: Tec<u8, Id8> = Default::default();
        (0..255).for_each(|i| {
            assert_eq!(entities.alloc(i), Id8::cast_from(i.into()));
        });

        entities.remove(Id8::cast_from(27));
        entities.remove(Id8::cast_from(254));
        entities.remove(Id8::cast_from(15));
        entities.remove(Id8::cast_from(252));
        entities.remove(Id8::cast_from(251));
        entities.remove(Id8::cast_from(253));

        let mut records_old = HashSet::new();
        let mut records_new = HashSet::new();

        entities.coalesce(|old_id, new_id| {
            records_old.insert(old_id);
            records_new.insert(new_id);

            // update all data that reference the old_id and replace them with new_id
        });

        assert_eq!(
            records_old,
            HashSet::from([Id8::cast_from(250), Id8::cast_from(249)])
        );
        assert_eq!(
            records_new,
            HashSet::from([Id8::cast_from(15), Id8::cast_from(27)])
        );
    }

    fn create_remove_end_1() -> Tec<u8, u8> {
        let mut entities: Tec<u8, u8> = Default::default();
        (0..255).for_each(|i| {
            assert_eq!(entities.alloc(i), i);
        });

        entities.remove(27);
        entities.remove(254);
        entities.remove(15);
        entities.remove(252);
        entities.remove(251);
        entities.remove(253);

        entities
    }

    fn create_remove_end_2() -> Tec<u8, u8> {
        let mut entities: Tec<_, _> = Default::default();
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

        entities
    }

    #[test]
    fn remove_base_case_1() {
        // test removing to empty

        let mut entities: Tec<u8, u8> = Default::default();
        assert_eq!(entities.alloc(23), 0);
        assert_eq!(entities.alloc(23), 1);
        assert_eq!(entities.len(), 2);

        // remove everything
        entities.remove(0);
        entities.remove(1);
        assert!(entities.is_empty());

        entities.alloc(23);
        assert_eq!(entities.len(), 1);
        entities.alloc(23);
        assert_eq!(entities.len(), 2);
    }

    #[test]
    fn remove_end_1() {
        let entities = create_remove_end_1();

        let (data, _) = entities
            .iter_with_id()
            .rev()
            .next()
            .expect("should have at least 1 item");
        assert_eq!(data, 250);
        assert_eq!(entities.len(), 249);
    }

    #[test]
    fn remove_end_2() {
        let entities = create_remove_end_2();

        let (data, _) = entities
            .iter_with_id()
            .rev()
            .next()
            .expect("should have at least 1 item");
        assert_eq!(data, 230);
        assert_eq!(entities.len(), 224);
    }

    #[test]
    fn remove_end_3() {
        let mut entities: Tec<u8, u8> = Default::default();
        entities.alloc(0);
        entities.remove(0);
    }

    #[test]
    fn coalesce_1() {
        let mut entities = create_remove_end_1();

        let mut records_old = HashSet::new();
        let mut records_new = HashSet::new();

        entities.coalesce(|old_id, new_id| {
            records_old.insert(old_id);
            records_new.insert(new_id);

            // update all data that reference the old_id and replace them with new_id
        });

        assert_eq!(records_old, HashSet::from([249, 250]));
        assert_eq!(records_new, HashSet::from([15, 27]));
    }

    #[test]
    fn coalesce_2() {
        let mut entities = create_remove_end_2();

        let mut records_old = HashSet::new();
        let mut records_new = HashSet::new();

        entities.coalesce(|old_id, new_id| {
            records_old.insert(old_id);
            records_new.insert(new_id);

            // update all data that reference the old_id and replace them with new_id
        });

        assert!(records_old.iter().all(|index| *index > 223));

        let unique_values: HashSet<_> = entities.iter_with_id().map(|(_, data)| *data).collect();
        assert_eq!(unique_values.len(), 224);

        let expected = HashSet::from([27, 15, 25, 35, 34, 30]);
        assert_eq!(records_new, expected); // reclaiming from the last-issued
    }

    #[test]
    #[should_panic(expected = "removing an item from an empty container")]
    fn remove_unallocated_element() {
        let mut tec = Tec::<u8>::default();
        tec.remove(12321);
    }

    #[test]
    #[should_panic(expected = "removing an item from an empty container")]
    fn index_overflow() {
        let mut tec = Tec::<u8>::default();
        tec.remove(12321);
    }

    #[test]
    #[should_panic(expected = "removing a dead item")]
    fn remove_dead_element() {
        let mut tec = Tec::default();
        tec.alloc(12);
        let id: u32 = tec.alloc(23);
        tec.alloc(23);

        tec.remove(id);
        tec.remove(id);
    }

    #[test]
    #[should_panic(expected = "exceed storage limit")]
    fn alloc_over_max_capacity() {
        let mut tec = Tec::<u8, u8>::default();
        (0..=u8::MAX).for_each(|val| {
            tec.alloc(val);
        });
    }

    #[test]
    fn it_works() {
        let mut tec = Tec::with_capacity(2);
        assert_eq!(tec.len(), 0);

        let e1 = 1212;
        let i1: u16 = tec.alloc(e1);
        assert_eq!(tec.len(), 1);
        assert_eq!(tec[i1], e1);

        let e2 = 31232;
        let i2 = tec.alloc(e2);
        assert_eq!(tec.len(), 2);
        assert_eq!(tec[i2], e2);

        tec.clear();
        assert_eq!(tec.len(), 0);

        let e1 = 1212;
        let i1 = tec.alloc(e1);
        assert_eq!(tec.len(), 1);
        assert_eq!(tec[i1], e1);

        let e2 = 31232;
        let i2 = tec.alloc(e2);
        assert_eq!(tec.len(), 2);
        assert_eq!(tec[i2], e2);
    }

    #[test]
    fn insert() {
        // test collect() & get & index & count
        let a = 12312;
        let b = 654645;
        let c = 0;
        let d = 123;
        let mut tec = Tec::<_, u8>::default();
        let a_id = tec.alloc(a);
        let b_id = tec.alloc(b);
        let c_id = tec.alloc(c);
        let d_id = tec.alloc(d);

        assert_eq!(tec.get(a_id).cloned(), Some(a));
        assert_eq!(tec.get(b_id).cloned(), Some(b));
        assert_eq!(tec.get(c_id).cloned(), Some(c));
        assert_eq!(tec.get(d_id).cloned(), Some(d));
        assert_eq!(tec[a_id], a);
        assert_eq!(tec[b_id], b);
        assert_eq!(tec[c_id], c);
        assert_eq!(tec[d_id], d);
        assert_eq!(tec.len(), 4);

        // test alloc()
        let e = 43243;
        let e_index = tec.alloc(e);
        assert_eq!(tec.len(), 5);
        assert_eq!(e_index, 4);
        assert_eq!(tec[e_index], e);

        let e_index = tec.alloc(e);
        assert_eq!(tec.len(), 6);
        assert_eq!(e_index, 5);
        assert_eq!(tec[e_index], e);
    }

    #[test]
    fn remove() {
        let mut tec = Tec::<_, u8>::default();

        (0..100u8).for_each(|val| {
            tec.alloc(val);
        });

        assert_eq!(tec.len(), 100);

        tec.remove(90);
        assert_eq!(tec.len(), 99);
        assert!(tec
            .iter()
            .take(90)
            .enumerate()
            .all(|(index, val)| index as u8 == *val));
        let temp: Vec<_> = tec.iter().skip(90).enumerate().collect();
        assert_eq!(temp.len(), 9);
        assert!(temp.iter().all(|&(index, val)| (index as u8) + 91 == *val));

        // insert at the dead slot
        let e1 = 123;
        let i1 = tec.alloc(e1);
        assert_eq!(i1, 90);
        assert_eq!(tec[i1], e1);
        assert_eq!(tec.len(), 100);

        // remove twice then insert
        tec.remove(20);
        tec.remove(32);
        assert_eq!(tec.len(), 98);

        let e2 = 124;
        let e3 = 125;
        let i2 = tec.alloc(e2);
        assert_eq!(tec.len(), 99);
        assert_eq!(tec[i2], e2);
        assert_eq!(i2, 32);

        let i3 = tec.alloc(e3);
        assert_eq!(tec.len(), 100);
        assert_eq!(tec[i3], e3);
        assert_eq!(i3, 20);
    }

    #[test]
    fn test_remove_then_fill() {
        // make sure Tec can reclaim 100 slots after removing 100 slots.

        let mut tec: Tec<u8, u8> = Default::default();

        for i in 0..255 {
            assert_eq!(i, tec.alloc(i));
        }

        for i in 50..150 {
            assert_eq!(i, tec.remove(i));
        }

        assert_eq!(tec.len(), 155);

        for i in 0..100 {
            tec.alloc(i + 50);
        }
    }

    #[test]
    #[should_panic = "exceed storage limit"]
    fn test_remove_then_fill_overflow() {
        // similar to test_remove_then_fill(), but this one alloc() one more item, causing a panic
        let mut tec: Tec<u8, u8> = Default::default();

        for i in 0..255 {
            assert_eq!(i, tec.alloc(i));
        }

        for i in 50..150 {
            assert_eq!(i, tec.remove(i));
        }

        assert_eq!(tec.len(), 155);

        for i in 0..100 {
            tec.alloc(i + 50);
        }

        tec.alloc(11);
    }

    #[test]
    fn test_remove() {
        let mut tec: Tec<usize> = Default::default();

        let total = 10;

        for i in 0..total {
            assert_eq!(i, tec.alloc(i));
        }

        for i in total..0 {
            assert_eq!(tec.len(), i);
            assert_eq!(i, tec.remove(i));
        }
    }

    #[test]
    fn test_remove2() {
        let mut tec: Tec<usize> = Default::default();

        let remove_items = [1, 4, 5, 3, 2, 0];

        for i in 0..remove_items.len() {
            assert_eq!(i, tec.alloc(i));
        }

        for item in remove_items {
            assert_eq!(item, tec.remove(item));
        }
    }

    #[test]
    fn test_remove3() {
        // edge-case: removing items in a way that eliminates all dead slots,
        //          so that the head would be pointing to an invalid element

        let mut tec: Tec<usize> = Default::default();

        let remove_items = [0, 3, 2, 1, 4];

        for i in 0..remove_items.len() {
            assert_eq!(i, tec.alloc(i));
        }

        for item in remove_items {
            assert_eq!(item, tec.remove(item));
        }
    }

    #[test]
    fn iter() {
        let mut entities = Tec::default();

        fn check_all(entities: &Tec<String>) {
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

        assert_eq!(entities.remove(1), "1".to_owned());
        check_all(&entities);

        assert_eq!(entities.remove(4), "4".to_owned());
        check_all(&entities);

        assert_eq!(entities.remove(5), "5".to_owned());
        check_all(&entities);

        assert_eq!(entities.remove(2), "2".to_owned());
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

        assert_eq!(data_with_id, entities.clone().into_iter_with_id().collect(),);

        entities
            .iter_mut_with_id()
            .for_each(|(_, value)| *value = format!("1{value}"));

        assert_eq!(
            HashSet::from([(3, "13".to_owned()), (0, "10".to_owned())]),
            entities
                .iter_with_id()
                .map(|(id, value)| (id, value.to_owned()))
                .collect(),
        );

        entities
            .iter_mut()
            .for_each(|value| *value = format!("1{value}"));

        assert_eq!(
            HashSet::from([(3, "113".to_owned()), (0, "110".to_owned())]),
            entities
                .iter_with_id()
                .map(|(id, value)| (id, value.to_owned()))
                .collect(),
        );
    }
}
