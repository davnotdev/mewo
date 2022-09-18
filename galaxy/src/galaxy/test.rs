use crate::prelude::*;

//  OG Test
//  `--,
//     v
//  https://github.com/davnotdev/mewotk/blob/ffc3675f90d807a6acd9252728e8306ad7a24afb/mewo_ecs/src/executor/straight.rs
//  Spawn 3 entities.
//  Each has a `Data`.
//  Additionally, one has With, and one has Without.
//  One system += 1 onto all Data.0.
//  One system += 1 onto all Data.1 with `With`.
//  One system += 1 onto all Data.2 without `Without`.
//  Expected result:
//  e0: (1, 0, 1)
//  e1: (1, 1, 1)
//  e2: (1, 0, 0)
#[test]
fn test_galaxy_og() {
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct Data(usize, usize, usize);
    impl Component for Data {
        fn mewo_component_storage_type() -> ComponentStorageType {
            ComponentStorageType::Special
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct With;
    impl Component for With {
        fn mewo_component_storage_type() -> ComponentStorageType {
            ComponentStorageType::Special
        }
    }

    #[derive(Debug, Clone, PartialEq)]
    struct Without;
    impl Component for Without {
        fn mewo_component_storage_type() -> ComponentStorageType {
            ComponentStorageType::Special
        }
    }

    struct BasicExec {
        ev_modify: EventModify,
        st_trans: Vec<StorageTransform>,
    }

    impl Executor for BasicExec {
        fn new() -> Self
        where
            Self: Sized,
        {
            BasicExec {
                ev_modify: (EventModify::new()),
                st_trans: (Vec::new()),
            }
        }
        fn get_event_modify(&self) -> &mut EventModify {
            unsafe { &mut *(&self.ev_modify as *const EventModify as *mut EventModify) }
        }
        fn get_storage_transforms(&self) -> &mut Vec<StorageTransform> {
            unsafe {
                &mut *(&self.st_trans as *const Vec<StorageTransform> as *mut Vec<StorageTransform>)
            }
        }

        fn get_all_event_modify(&mut self) -> &mut [EventModify] {
            std::slice::from_mut(&mut self.ev_modify)
        }

        fn get_all_storage_transforms(&mut self) -> &mut [Vec<StorageTransform>] {
            std::slice::from_mut(&mut self.st_trans)
        }

        fn clear_all_storage_transforms(&mut self) {
            self.st_trans.clear();
        }
    }

    let mut galaxy = Galaxy::<BasicExec>::new();
    let a = galaxy.insert_entity().insert(Data::default()).get_entity();
    let b = galaxy
        .insert_entity()
        .insert(Data::default())
        .insert(With)
        .get_entity();
    let c = galaxy
        .insert_entity()
        .insert(Data::default())
        .insert(Without)
        .get_entity();
    galaxy.update();

    //  System A
    for data in galaxy.query::<&mut Data>().iter() {
        data.0 += 1;
    }

    //  System B
    for data in galaxy.query::<&mut Data>().with::<With>().iter() {
        data.1 += 1;
    }

    //  System C
    for data in galaxy.query::<&mut Data>().without::<Without>().iter() {
        data.2 += 1;
    }

    galaxy.update();

    assert!(galaxy.get_entity(a).unwrap().get::<&Data>().unwrap().get() == &Data(1, 0, 1));
    assert!(galaxy.get_entity(b).unwrap().get::<&Data>().unwrap().get() == &Data(1, 1, 1));
    assert!(galaxy.get_entity(c).unwrap().get::<&Data>().unwrap().get() == &Data(1, 0, 0));
}
