use super::*;

pub struct StraightExecutor {
    sys: Vec<(SystemWish, BoxedSystem, SystemData)>,
    commands: WorldCommands,
    global_wish: GlobalWish,
}

impl Executor for StraightExecutor {
    fn create(world: &World, sys: Vec<(BoxedSystem, SystemData)>) -> Self {
        let mut self_sys = Vec::new();
        let global_wish = GlobalWish::create(world.get_component_manager());
        for (sys, sys_data) in sys {
            self_sys.push((
                SystemWish::create(world, &global_wish, &sys_data),
                sys, sys_data, 
            ));
        }
        StraightExecutor {
            sys: self_sys,
            commands: WorldCommands::create(),
            global_wish,
        }
    }

    fn execute(&mut self, world: &mut World) {
        let world_changed = world.is_world_changed();
        if world_changed {
            self.global_wish.recreate_slices(world.get_component_manager())
        }
        for (wish, sys, sys_data) in self.sys.iter_mut() {
            if world_changed {
                wish.update_index_buf(world, sys_data);
            }
            let inst = WishInstance::create(&wish, &self.global_wish);
            let args = SystemArgs {
                rmgr: world.get_resource_manager(),
                cmds: &mut self.commands,
            };
            sys.call(&inst, args);
        }
        world.reset_world_changed();

        let (spawns, removes, modifies) = self.commands.get_entity_commands();
        for (e, modify) in modifies {
            world.modify_entity(*e, modify);
        }
        for remove_i in 0..removes.get_len() {
            if removes.get(remove_i).unwrap() {
                world.remove_entity(Entity::from_id(remove_i as u32));
            }
        }
        for spawn in spawns {
            world.insert_entity(
                if let Some(spawn) = spawn {
                    Some(spawn)
                } else {
                    None
                }
            );
        }
        let modifies = self.commands.get_resource_commands();
        for modify in modifies {
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
    use crate::{ 
        Wish, 
        Write,
        Component, 
        EntityWrapper,
        EntityModifyCallback,
        GenericEntityModifyCallback,
    };
    #[derive(Debug, Clone, PartialEq)]
    struct Data(usize, usize, usize);
    impl Component for Data {}
    #[derive(Debug, Clone, PartialEq)]
    struct With;
    impl Component for With {}
    #[derive(Debug, Clone, PartialEq)]
    struct Without;
    impl Component for Without {}

    let mut world = World::create();
    let component_manager = world.get_mut_component_manager();
    component_manager.register_component_type::<Data>().unwrap();
    component_manager.register_component_type::<With>().unwrap();
    component_manager.register_component_type::<Without>().unwrap();
    
    fn sysall(q: Wish<Write<Data>>, _args: SystemArgs) {
        for (data, _e) in q.write::<Data>() {
            data.0 += 1;
        }
    }
    let sysall = System(sysall);
    fn syswith(q: Wish<Write<Data, With, ()>>, _args: SystemArgs) {
        for (data, _e) in q.write::<Data>() {
            data.1 += 1;
        }
    }
    let syswith = System(syswith);
    fn syswithout (q: Wish<Write<Data, (), Without>>, _args: SystemArgs) {
        for (data, _e) in q.write::<Data>() {
            data.2 += 1;
        }
    }
    let syswithout = System(syswithout);
    
    let callback : Box<dyn GenericEntityModifyCallback> = Box::new(EntityModifyCallback(|mut e: EntityWrapper| {
        e.insert_component::<Data>(Data(0, 0, 0)); 
    }));
    world.insert_entity(Some(&callback));
    let callback : Box<dyn GenericEntityModifyCallback> = Box::new(EntityModifyCallback(move |mut e: EntityWrapper| {
        e.insert_component::<Data>(Data(0, 0, 0)); 
        e.insert_component::<With>(With);
    }));
    world.insert_entity(Some(&callback));

    let callback : Box<dyn GenericEntityModifyCallback> = Box::new(EntityModifyCallback(|mut e: EntityWrapper| {
        e.insert_component::<Data>(Data(0, 0, 0)); 
        e.insert_component::<Without>(Without);
    }));
    world.insert_entity(Some(&callback));

    let sysall_info = SystemData::from_query_type(&world, &sysall.get_wish_info());
    let syswith_info = SystemData::from_query_type(&world, &syswith.get_wish_info());
    let syswithout_info = SystemData::from_query_type(&world, &syswithout.get_wish_info());
    let mut exec = StraightExecutor::create(&world, vec![
        (Box::new(sysall), sysall_info),
        (Box::new(syswith), syswith_info),
        (Box::new(syswithout), syswithout_info),
    ]);
    exec.execute(&mut world);

    assert_eq!(world.get_component_with_entity::<Data>(Entity { id: 0 }), &Data(1, 0, 1));
    assert_eq!(world.get_component_with_entity::<Data>(Entity { id: 1 }), &Data(1, 1, 1));
    assert_eq!(world.get_component_with_entity::<Data>(Entity { id: 2 }), &Data(1, 0, 0));
}

