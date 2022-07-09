use super::{
    component::Component,
    entity::EntityBus,
    event::{Event, EventBus},
    resource::{Resource, ResourceBus},
    system::{SystemBus, SystemFunction},
    wish::{Wish, WishAccesses, WishEvent, WishFilters},
};
use mewo_ecs::{
    ComponentTypeEntry, EventTypeEntry, RawPlugin, ResourceTypeEntry, SystemBuilder, ValueClone,
    ValueDrop,
};

pub struct PluginBuilder {
    raw_plugin: RawPlugin,
}

impl PluginBuilder {
    pub fn create<SELFP: Plugin>() -> Self {
        PluginBuilder {
            raw_plugin: RawPlugin {
                name: SELFP::name().to_string(),
                deps: Vec::new(),
                events: Vec::new(),
                systems: Vec::new(),
                resources: Vec::new(),
                components: Vec::new(),
            },
        }
    }

    pub fn dep<P: Plugin>(self) -> Self {
        self.raw_dep(P::name().to_string())
    }

    pub fn raw_dep(mut self, name: String) -> Self {
        self.raw_plugin.deps.push(name);
        self
    }

    pub fn comp<C: Component>(self) -> Self {
        let size = C::component_size();
        self.raw_comp(ComponentTypeEntry {
            name: C::component_name().to_string(),
            size,
            hash: C::component_hash(),
            drop: ValueDrop::create(C::component_drop_callback()),
            clone: ValueClone::create(C::component_clone_callback()),
        })
    }

    pub fn raw_comp(mut self, entry: ComponentTypeEntry) -> Self {
        self.raw_plugin.components.push(entry);
        self
    }

    pub fn sys<WE, WA, WF>(self, function: SystemFunction<WE, WA, WF>) -> Self
    where
        WE: 'static + WishEvent,
        WA: 'static + WishAccesses,
        WF: 'static + WishFilters,
    {
        self.raw_sys(SystemBuilder::create(
            std::any::type_name::<SystemFunction<WE, WA, WF>>().to_string(),
            WE::hash(),
            WA::accesses(),
            WF::filters(),
            Box::new(
                move |ev, ev_insert, rcmgr, rc_modify, entity_transformer, galaxy, akid| {
                    let amgr = galaxy.get_archetype_manager();
                    let ctymgr = galaxy.get_component_type_manager();
                    let count = amgr.get_access_count(akid);
                    if Wish::<WE, WA, WF>::is_empty() {
                        (function)(
                            SystemBus::create(
                                EntityBus::create(entity_transformer),
                                EventBus::create(ev_insert),
                                ResourceBus::create(rcmgr, rc_modify),
                            ),
                            Wish::<WE, WA, WF>::create(ev, ctymgr, None),
                        );
                    } else {
                        //  Would be nice if you could move to the next access instead of spinning.
                        for idx in 0..count {
                            loop {
                                if let Some(access) = amgr.try_access(akid, idx).unwrap() {
                                    let wish =
                                        Wish::<WE, WA, WF>::create(ev, ctymgr, Some(&access));
                                    (function)(
                                        SystemBus::create(
                                            EntityBus::create(entity_transformer),
                                            EventBus::create(ev_insert),
                                            ResourceBus::create(rcmgr, rc_modify),
                                        ),
                                        wish,
                                    );
                                    break;
                                }
                                std::hint::spin_loop();
                            }
                        }
                    }
                },
            ),
        ))
    }

    pub fn raw_sys(mut self, mut sys: SystemBuilder) -> Self {
        sys.set_plugin_name(self.raw_plugin.name.clone());
        self.raw_plugin.systems.push(sys);
        self
    }

    pub fn raw_event(mut self, ev: EventTypeEntry) -> Self {
        self.raw_plugin.events.push(ev);
        self
    }

    pub fn event<E: Event>(self) -> Self {
        self.raw_event(EventTypeEntry {
            size: E::event_size(),
            name: E::event_name(),
            hash: E::event_hash(),
            drop: ValueDrop::create(E::event_drop_callback()),
        })
    }

    pub fn raw_resource(mut self, rc: ResourceTypeEntry) -> Self {
        self.raw_plugin.resources.push(rc);
        self
    }

    pub fn resource<R: Resource>(self) -> Self {
        self.raw_resource(ResourceTypeEntry {
            size: R::resource_size(),
            name: R::resource_name(),
            hash: R::resource_hash(),
            drop: ValueDrop::create(R::resource_drop_callback()),
        })
    }

    pub fn build(self) -> RawPlugin {
        self.raw_plugin
    }
}

pub trait Plugin {
    fn name() -> &'static str;
    fn plugin(pb: PluginBuilder) -> PluginBuilder;
}
