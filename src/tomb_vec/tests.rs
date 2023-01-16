#[cfg(test)]
mod tests {

    use std::collections::HashSet;

    use crate::Tec;

    fn create_remove_end_1() -> Tec<u8, u8> {
        let mut entities: Tec<_, _> = Default::default();
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
        print!("1");
        entities.remove(0);
        print!("2");
        entities.remove(1);
        print!("3");

        assert_eq!(entities.alloc(23), 0);
        assert_eq!(entities.alloc(23), 1);
    }

    #[test]
    fn remove_end_1() {
        let entities = create_remove_end_1();

        let (data, _) = entities
            .iter()
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
            .iter()
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

        assert_eq!(records_old, HashSet::from([15, 27]));
        assert_eq!(records_new, HashSet::from([249, 250]));
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

        let expected = HashSet::from([27, 15, 25, 35, 34, 30]);
        assert_eq!(records_old, expected); // reclaiming from the last-issued

        let unique_values: HashSet<_> = entities.iter().map(|(_, data)| *data).collect();
        assert_eq!(unique_values.len(), 224);
        assert!(records_new.iter().all(|index| *index > 223));
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
            .all(|(index, (_, val))| index as u8 == *val));
        let temp: Vec<_> = tec.iter().skip(90).enumerate().collect();
        assert_eq!(temp.len(), 9);
        assert!(temp
            .iter()
            .all(|&(index, (_, val))| (index as u8) + 91 == *val));

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
}
