use crate::{Component, EntityWrapper, SystemDataSet, Wish, With, Without, World};

struct ReadComponent;
impl Component for ReadComponent {}

struct WriteComponent;
impl Component for WriteComponent {}

struct OtherComponent;
impl Component for OtherComponent {}

struct WithComponent;
impl Component for WithComponent {}

struct WithoutComponent;
impl Component for WithoutComponent {}

#[test]
fn test_filter_single() {
    let mut mock_world = World::create();
    let cmgr = mock_world.get_mut_component_manager();
    cmgr.register_component_type::<ReadComponent>().unwrap();
    cmgr.register_component_type::<WriteComponent>().unwrap();

    let e0 = mock_world.insert_entity();
    let e1 = mock_world.insert_entity();
    let e2 = mock_world.insert_entity();
    EntityWrapper::from_entity(e0, &mut mock_world).insert_component(ReadComponent);
    EntityWrapper::from_entity(e1, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent);
    EntityWrapper::from_entity(e2, &mut mock_world).insert_component(WriteComponent);

    let system_data_set =
        SystemDataSet::from_wish_data(&mock_world, &Wish::<&ReadComponent, ()>::get_wish_data())
            .unwrap();
    assert_eq!(system_data_set.match_entity(&mock_world, e0), Ok(true));
    assert_eq!(system_data_set.match_entity(&mock_world, e1), Ok(true));
    assert_eq!(system_data_set.match_entity(&mock_world, e2), Ok(false));
}

#[test]
fn test_filter_multi() {
    let mut mock_world = World::create();
    let cmgr = mock_world.get_mut_component_manager();
    cmgr.register_component_type::<ReadComponent>().unwrap();
    cmgr.register_component_type::<WriteComponent>().unwrap();
    cmgr.register_component_type::<OtherComponent>().unwrap();

    let e0 = mock_world.insert_entity();
    let e1 = mock_world.insert_entity();
    let e2 = mock_world.insert_entity();
    let e3 = mock_world.insert_entity();
    let e4 = mock_world.insert_entity();
    EntityWrapper::from_entity(e0, &mut mock_world).insert_component(ReadComponent);
    EntityWrapper::from_entity(e1, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent);
    EntityWrapper::from_entity(e2, &mut mock_world).insert_component(WriteComponent);
    EntityWrapper::from_entity(e3, &mut mock_world)
        .insert_component(WriteComponent)
        .insert_component(OtherComponent);
    EntityWrapper::from_entity(e4, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent)
        .insert_component(OtherComponent);

    let system_data_set = SystemDataSet::from_wish_data(
        &mock_world,
        &Wish::<(&ReadComponent, &mut WriteComponent), ()>::get_wish_data(),
    )
    .unwrap();
    assert_eq!(system_data_set.match_entity(&mock_world, e0), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e1), Ok(true));
    assert_eq!(system_data_set.match_entity(&mock_world, e2), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e3), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e4), Ok(true));
}
#[test]
fn test_filter_multi_with_without() {
    let mut mock_world = World::create();
    let cmgr = mock_world.get_mut_component_manager();
    cmgr.register_component_type::<ReadComponent>().unwrap();
    cmgr.register_component_type::<WriteComponent>().unwrap();
    cmgr.register_component_type::<OtherComponent>().unwrap();
    cmgr.register_component_type::<WithComponent>().unwrap();
    cmgr.register_component_type::<WithoutComponent>().unwrap();

    let e0 = mock_world.insert_entity();
    let e1 = mock_world.insert_entity();
    let e2 = mock_world.insert_entity();
    let e3 = mock_world.insert_entity();
    let e4 = mock_world.insert_entity();
    let e5 = mock_world.insert_entity();
    let e6 = mock_world.insert_entity();
    let e7 = mock_world.insert_entity();
    EntityWrapper::from_entity(e0, &mut mock_world).insert_component(ReadComponent);
    EntityWrapper::from_entity(e1, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent);
    EntityWrapper::from_entity(e2, &mut mock_world).insert_component(WriteComponent);
    EntityWrapper::from_entity(e3, &mut mock_world)
        .insert_component(WriteComponent)
        .insert_component(OtherComponent);
    EntityWrapper::from_entity(e4, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent)
        .insert_component(OtherComponent);
    EntityWrapper::from_entity(e5, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent)
        .insert_component(WithComponent);
    EntityWrapper::from_entity(e6, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent)
        .insert_component(OtherComponent)
        .insert_component(WithComponent);
    EntityWrapper::from_entity(e7, &mut mock_world)
        .insert_component(ReadComponent)
        .insert_component(WriteComponent)
        .insert_component(OtherComponent)
        .insert_component(WithComponent)
        .insert_component(WithoutComponent);

    let system_data_set = SystemDataSet::from_wish_data(
        &mock_world,
        &Wish::<
            (&ReadComponent, &mut WriteComponent),
            (With<WithComponent>, Without<WithoutComponent>),
        >::get_wish_data(),
    )
    .unwrap();
    assert_eq!(system_data_set.match_entity(&mock_world, e0), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e1), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e2), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e3), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e4), Ok(false));
    assert_eq!(system_data_set.match_entity(&mock_world, e5), Ok(true));
    assert_eq!(system_data_set.match_entity(&mock_world, e6), Ok(true));
    assert_eq!(system_data_set.match_entity(&mock_world, e7), Ok(false));
}
