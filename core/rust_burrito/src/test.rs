pub use crate::*;

//  OG Test
//  `--,
//     v
//  https://github.com/davnotdev/mewotk/blob/ffc3675f90d807a6acd9252728e8306ad7a24afb/mewo_ecs/src/executor/straight.rs
//  Spawn 3 entities.
//  Each has a `Data`.
//  Additionally, one has WithC, and one has WithoutC.
//  One system += 1 onto all Data.0.
//  One system += 1 onto all Data.1 with `WithC`.
//  One system += 1 onto all Data.2 without `WithoutC`.
//  Expected result:
//  e0: (1, 0, 1)
//  e1: (1, 1, 1)
//  e2: (1, 0, 0)
#[test]
fn test_rust_burrito_og() {
    #[derive(Default, Debug, Clone, Copy, PartialEq)]
    struct Data(usize, usize, usize);
    impl Component for Data {}

    #[derive(Debug, Clone, PartialEq)]
    struct WithC;
    impl Component for WithC {}

    #[derive(Debug, Clone, PartialEq)]
    struct WithoutC;
    impl Component for WithoutC {}

    struct MyPlugin;

    impl Plugin for MyPlugin {
        fn name() -> &'static str {
            "test_my_plugin"
        }
        fn plugin(pb: PluginBuilder) -> PluginBuilder {
            pb.startup(startup_system)
                .update(system_a)
                .update(system_b)
                .update(system_c)
        }
    }

    fn startup_system(mut sb: SystemBus<(), ()>) -> Option<()> {
        sb.entities.spawn().insert(Data::default());
        sb.entities.spawn().insert(Data::default()).insert(WithC);
        sb.entities.spawn().insert(Data::default()).insert(WithoutC);
        Some(())
    }

    fn system_a(sb: SystemBus<&mut Data, ()>) {
        for data in sb.components.iter() {
            data.0 += 1;
        }
    }

    fn system_b(sb: SystemBus<&mut Data, With<WithC>>) {
        for data in sb.components.iter() {
            data.1 += 1;
        }
    }

    fn system_c(sb: SystemBus<&mut Data, Without<WithoutC>>) {
        for data in sb.components.iter() {
            data.2 += 1;
        }
    }

    let galaxy = Galaxy::create();
    galaxy.plugin(MyPlugin::build_plugin(&galaxy));
    let (mut runtime, mut exec) = galaxy.runtime::<StraightExecutor>();
    runtime.tick(&mut exec);

    use mewo_ecs::Entity;

    assert_eq!(
        unsafe {
            *(runtime
                .get_archetype_manager()
                .find_component(0, Entity::from_id(0)) as *const Data)
        },
        Data(1, 0, 1)
    );

    assert_eq!(
        unsafe {
            *(runtime
                .get_archetype_manager()
                .find_component(0, Entity::from_id(2)) as *const Data)
        },
        Data(1, 1, 1)
    );

    assert_eq!(
        unsafe {
            *(runtime
                .get_archetype_manager()
                .find_component(0, Entity::from_id(1)) as *const Data)
        },
        Data(1, 0, 0)
    );

    //  Wait, why aren't the entities in order??
    //  Swap remove is why.
    //  `while let Some(entity) = transform.entites.swap_remove(0) { .. }`
    //  Let the above pseudo-code sink in.
}
