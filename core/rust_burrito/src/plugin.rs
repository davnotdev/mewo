use super::{
    component::Component,
    entity::EntityBus,
    event::{Event, EventBus},
    params::{
        ComponentAccesses, ComponentFilters, Components, EventAccess, Events, SystemFunction,
    },
    resource::{Resource, ResourceBus},
    system::SystemBus,
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

    pub fn sys<EA, CA, CF>(self, function: SystemFunction<EA, CA, CF>) -> Self
    where
        EA: 'static + EventAccess,
        CA: 'static + ComponentAccesses,
        CF: 'static + ComponentFilters,
    {
        self.raw_sys(SystemBuilder::create(
            std::any::type_name::<SystemFunction<EA, CA, CF>>().to_string(),
            EA::hash(),
            CA::accesses(),
            CF::filters(),
            Box::new(
                move |galaxy, ev, rcmgr, ev_insert, transformer, access, idx, count| {
                    (function)(
                        SystemBus::create(
                            EntityBus::create(transformer),
                            EventBus::create(ev_insert),
                            ResourceBus::create(rcmgr),
                            idx,
                            count,
                        ),
                        Events::create(ev),
                        Components::create(galaxy.get_component_type_manager(), &access),
                    );
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
            name: R::resource_name(),
            hash: R::resource_hash(),
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
