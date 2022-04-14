use super::*;

pub struct StraightExecutor {
    sys: Vec<(Gift, System, SantaClaus)>,
    commands: WorldCommands,
    global_gift: GlobalGift,
}

impl Executor for StraightExecutor {
    fn create(world: &World, sys: Vec<(System, SantaClaus)>) -> Self {
        let mut self_sys = Vec::new();
        let global_gift = GlobalGift::create(world.get_component_manager());
        for (sys, santa) in sys {
            self_sys.push((
                Gift::create(world, &global_gift, &santa),
                sys, santa, 
            ));
        }
        StraightExecutor {
            sys: self_sys,
            commands: WorldCommands::create(),
            global_gift,
        }
    }

    fn execute(&mut self, world: &mut World) {
        let world_changed = world.is_world_changed();
        if world_changed {
            self.global_gift.recreate_slices(world.get_component_manager())
        }
        for (gift, sys, santa) in self.sys.iter_mut() {
            if world_changed {
                gift.update_index_buf(world, santa);
            }
            let mut inst = GiftInstance::create(&gift, &self.global_gift);
            (sys)(&mut inst, &mut self.commands);
        }
        world.reset_world_changed();

        let (spawns, removes, modifies) = self.commands.get_entity_commands();
        for (e, modify) in modifies {
            world.modify_entity(*e, *modify);
        }
        for remove_i in 0..removes.get_len() {
            if removes.get(remove_i).unwrap() {
                world.remove_entity(Entity::from_id(remove_i as u32));
            }
        }
        for spawn in spawns {
            world.insert_entity(*spawn);
        }
        let modifies = self.commands.get_resource_commands();
        for modify in modifies {
            world.modify_resources(*modify);
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
    #[derive(Debug, Clone, PartialEq)]
    struct Data(usize, usize, usize);
    #[derive(Debug, Clone, PartialEq)]
    struct With;
    #[derive(Debug, Clone, PartialEq)]
    struct Without;

    let mut world = World::create();
    let component_manager = world.get_mut_component_manager();
    component_manager.register_component_type::<Data>().unwrap();
    component_manager.register_component_type::<With>().unwrap();
    component_manager.register_component_type::<Without>().unwrap();
    
    let sysall = |gift: &mut GiftInstance, _cmd: &mut WorldCommands| {
        for (data, _e) in gift.write::<Data>() {
            data.0 += 1;
        }
    };
    let sysall_santa = SantaClaus::wishlist(&world)
        .writes(vec![0], None, None)
        .finish();
    let syswith = |gift: &mut GiftInstance, _cmd: &mut WorldCommands| {
        for (data, _e) in gift.write::<Data>() {
            data.1 += 1;
        }
    };
    let syswith_santa = SantaClaus::wishlist(&world)
        .writes(vec![0], Some(vec![1]), None)
        .finish();
    let syswithout = |gift: &mut GiftInstance, _cmd: &mut WorldCommands| {
        for (data, _e) in gift.write::<Data>() {
            data.2 += 1;
        }
    };
    let syswithout_santa = SantaClaus::wishlist(&world)
        .writes(vec![0], None, Some(vec![2]))
        .finish();

    world.insert_entity(Some(|mut e| {
        e.insert_component::<Data>(Data(0, 0, 0)); 
    }));
    world.insert_entity(Some(|mut e| {
        e.insert_component::<Data>(Data(0, 0, 0)); 
        e.insert_component::<With>(With);
    }));
    world.insert_entity(Some(|mut e| {
        e.insert_component::<Data>(Data(0, 0, 0)); 
        e.insert_component::<Without>(Without);
    }));

    let mut exec = StraightExecutor::create(&world, vec![
        (sysall, sysall_santa),
        (syswith, syswith_santa),
        (syswithout, syswithout_santa),
    ]);
    exec.execute(&mut world);

    assert_eq!(world.get_component_with_entity::<Data>(Entity { id: 0 }), &Data(1, 0, 1));
    assert_eq!(world.get_component_with_entity::<Data>(Entity { id: 1 }), &Data(1, 1, 1));
    assert_eq!(world.get_component_with_entity::<Data>(Entity { id: 2 }), &Data(1, 0, 0));
}

