use mewo_ecs::*;

#[derive(Clone)]
pub struct Event;

fn remove_events_sys(w: &World) -> SystemData {
    (remove_events, SantaClaus::wishlist(w)
        .reads(vec![w.component::<Event>()], None, None)
        .finish())
}

fn remove_events(gift: &mut GiftInstance, cmds: &mut WorldCommands) {
    for (_event, e) in gift.read::<Event>() {
        cmds.remove_entity(e);
    }
}

pub struct EventPlugin;

impl EventPlugin {
    pub fn name() -> &'static str {
        "mewo_tk_event"
    }

    pub fn plugin(pb: &mut PluginBuilder) {
        pb
            .component::<Event>()
            .sys(remove_events_sys)
        ;
    }
}

