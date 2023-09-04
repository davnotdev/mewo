use crate::*;

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
    impl CheapComponent for Data {}
    impl GenericComponent for Data {
        fn mewo_component_duplicate() -> ValueDuplicate {
            <Data as CheapComponent>::mewo_component_duplicate()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct With;
    impl CheapComponent for With {}
    impl GenericComponent for With {
        fn mewo_component_duplicate() -> ValueDuplicate {
            <With as CheapComponent>::mewo_component_duplicate()
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Without;
    impl CheapComponent for Without {}
    impl GenericComponent for Without {
        fn mewo_component_duplicate() -> ValueDuplicate {
            <Without as CheapComponent>::mewo_component_duplicate()
        }
    }

    let mut galaxy = Galaxy::new();
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

    let a = galaxy.get_entity(a).unwrap().get::<&Data>().unwrap().get();
    let b = galaxy.get_entity(b).unwrap().get::<&Data>().unwrap().get();
    let c = galaxy.get_entity(c).unwrap().get::<&Data>().unwrap().get();
    assert_eq!(a, &Data(1, 0, 1));
    assert_eq!(b, &Data(1, 1, 1));
    assert_eq!(c, &Data(1, 0, 0));
}
