use super::*;

pub struct StraightExecutor {
    commands: WorldCommands,
    systems: Vec<(BoxedSystem, Vec<SystemDataSet>, Vec<SystemDataSetInstance>)>,
}

impl Executor for StraightExecutor {
    fn create(world: &World, systems: Vec<(BoxedSystem, Vec<SystemDataSet>)>) -> Self {
        StraightExecutor {
            systems: {
                let mut self_systems = Vec::with_capacity(systems.len());
                for (sys, sets) in systems.into_iter() {
                    let mut insts = Vec::new();
                    for data in sets.iter() {
                        let inst = SystemDataSetInstance::create(world, &data).unwrap();
                        insts.push(inst);
                    }
                    self_systems.push((sys, sets, insts));
                }
                self_systems
            },
            commands: WorldCommands::create(),
        }
    }

    fn run_systems(&mut self, world: &mut World) {
        for (sys, _sets, insts) in self.systems.iter() {
            sys.call(world, &mut self.commands, insts);
        }
    }

    fn run_commands(&mut self, world: &mut World) {
        for (emh, callback) in self.commands.entity_cmds.iter_mut() {
            let e = match emh {
                EntityModifyHandle::Spawn => world.insert_entity(),
                EntityModifyHandle::Entity(e) => *e,
            };
            if let Some(callback) = callback {
                callback.call(e, world);
            };
            for (_sys, sets, insts) in self.systems.iter_mut() {
                for (inst, data) in insts.iter_mut().zip(sets.iter()) {
                    inst.any_entity_modify(&world, &data, e);
                }
            }
        }
        for e in self.commands.entity_removes.iter() {
            for (_sys, _sets, insts) in self.systems.iter_mut() {
                for inst in insts {
                    inst.any_entity_remove(*e);
                }
            }
            world.remove_entity(*e).unwrap();
        }
        for modify in self.commands.resource_modifies.iter() {
            world.modify_resources(modify);
        }

        self.commands.flush();
    }
}

//  spawn 3 entities
//  each has a Data
//  one system += 1 onto all Data.0
//  one system += 1 onto all Data.1 with With
//  one system += 1 onto all Data.2 without Without
//  expected result:
//  e0: (1, 0, 1)
//  e1: (1, 1, 1)
//  e2: (1, 0, 0)
#[test]
fn test_straight_executor() {
    use crate::{Component, EntityWrapper, SystemBuilder, Wish, With, Without};
    #[derive(Debug, Clone, PartialEq)]
    struct Data(usize, usize, usize);
    impl Component for Data {}
    #[derive(Debug, Clone, PartialEq)]
    struct WithComponent;
    impl Component for WithComponent {}
    #[derive(Debug, Clone, PartialEq)]
    struct WithoutComponent;
    impl Component for WithoutComponent {}

    let mut world = World::create();
    let component_manager = world.get_mut_component_manager();
    component_manager.register_component_type::<Data>().unwrap();
    component_manager
        .register_component_type::<WithComponent>()
        .unwrap();
    component_manager
        .register_component_type::<WithoutComponent>()
        .unwrap();

    fn sysall(_args: &mut SystemArgs, q: Wish<&mut Data, ()>) {
        for mut data in q.iter() {
            data.0 += 1;
        }
    }
    fn syswith(_args: &mut SystemArgs, q: Wish<&mut Data, With<WithComponent>>) {
        for mut data in q.iter() {
            data.1 += 1;
        }
    }
    fn syswithout(_args: &mut SystemArgs, q: Wish<&mut Data, Without<WithoutComponent>>) {
        for (_e, mut data) in q.eiter() {
            data.2 += 1;
        }
    }

    //  *shocked pikachu* they compile...
    fn syscompiletest0(_args: &mut SystemArgs, q: Wish<(&mut Data, &WithComponent), ()>) {
        for (_data, _with) in q.iter() {}
    }
    fn syscompiletest1(
        _args: &mut SystemArgs,
        q: (Wish<&mut Data, ()>, Wish<&mut WithComponent, ()>),
    ) {
        let (datas, withs) = q;
        for (_e, _data) in datas.eiter() {}
        for (_e, _with) in withs.eiter() {}
    }

    let e = world.insert_entity();
    let mut e = EntityWrapper::from_entity(e, &mut world);
    e.insert_component(Data(0, 0, 0));
    let e = world.insert_entity();
    let mut e = EntityWrapper::from_entity(e, &mut world);
    e.insert_component(Data(0, 0, 0));
    e.insert_component(WithComponent);
    let e = world.insert_entity();
    let mut e = EntityWrapper::from_entity(e, &mut world);
    e.insert_component(Data(0, 0, 0));
    e.insert_component(WithoutComponent);

    let mut systems = SystemBuilder::create();
    systems
        .sys(&world, sysall)
        .sys(&world, syswith)
        .sys(&world, syswithout)
        .sys(&world, syscompiletest0)
        .sys(&world, syscompiletest1);
    let mut exec = StraightExecutor::create(&world, systems.consume());
    exec.run_systems(&mut world);

    assert_eq!(
        world
            .get_mut_component_manager()
            .get_boxed_storage_of::<Data>()
            .unwrap()
            .get_storage::<Data>()
            .unwrap()
            .get_component_with_entity(Entity { id: 0 }),
        Ok(&Data(1, 0, 1))
    );
    assert_eq!(
        world
            .get_mut_component_manager()
            .get_boxed_storage_of::<Data>()
            .unwrap()
            .get_storage::<Data>()
            .unwrap()
            .get_component_with_entity(Entity { id: 1 }),
        Ok(&Data(1, 1, 1))
    );
    assert_eq!(
        world
            .get_mut_component_manager()
            .get_boxed_storage_of::<Data>()
            .unwrap()
            .get_storage::<Data>()
            .unwrap()
            .get_component_with_entity(Entity { id: 2 }),
        Ok(&Data(1, 0, 0))
    );
}
