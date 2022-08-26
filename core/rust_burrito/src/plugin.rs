use super::{
    component::{ComponentAccesses, ComponentFilters, Components},
    entity::EntityBus,
    event::EventBus,
    resource::ResourceBus,
    system::{EarlySystemFunction, SystemBus, SystemFunction},
};
use mewo_ecs::{EarlySystemPhase, Galaxy, RawPlugin, SystemBuilder};

pub struct PluginBuilder<'galaxy> {
    galaxy: &'galaxy Galaxy,
    raw_plugin: RawPlugin,
}

impl<'galaxy> PluginBuilder<'galaxy> {
    pub fn create<SELFP: Plugin>(galaxy: &'galaxy Galaxy) -> Self {
        PluginBuilder {
            galaxy,
            raw_plugin: RawPlugin {
                name: SELFP::name().to_string(),
                systems: Vec::new(),
            },
        }
    }

    fn generic_early<CA, CF>(
        self,
        function: EarlySystemFunction<CA, CF>,
        phase: EarlySystemPhase,
    ) -> Self
    where
        CA: 'static + ComponentAccesses,
        CF: 'static + ComponentFilters,
    {
        {
            let mut ctymgr = self.galaxy.get_component_type_manager().write().unwrap();
            CA::maybe_register(&mut ctymgr);
            CF::maybe_register(&mut ctymgr);
        }
        self.raw_sys(SystemBuilder::create(
            std::any::type_name::<EarlySystemFunction<CA, CF>>().to_string(),
            CA::accesses(),
            CF::filters(),
            mewo_ecs::SystemFunction(Box::new(
                move |galaxy, ev_insert, entity_transformer, access_key| {
                    (function)(SystemBus::create(
                        EntityBus::create(galaxy.get_component_type_manager(), entity_transformer),
                        EventBus::create(galaxy.get_event_manager(), ev_insert),
                        ResourceBus::create(galaxy.get_resource_manager()),
                        Components::create(
                            galaxy.get_component_type_manager(),
                            galaxy.get_archetype_manager(),
                            access_key,
                        ),
                    ))
                    .is_some()
                },
            )),
            Some(phase),
        ))
    }

    pub fn bootstrap<CA, CF>(self, function: EarlySystemFunction<CA, CF>) -> Self
    where
        CA: 'static + ComponentAccesses,
        CF: 'static + ComponentFilters,
    {
        Self::generic_early(self, function, EarlySystemPhase::Bootstrap)
    }

    pub fn startup<CA, CF>(self, function: EarlySystemFunction<CA, CF>) -> Self
    where
        CA: 'static + ComponentAccesses,
        CF: 'static + ComponentFilters,
    {
        Self::generic_early(self, function, EarlySystemPhase::Startup)
    }

    pub fn update<CA, CF, O>(self, function: SystemFunction<CA, CF, O>) -> Self
    where
        CA: 'static + ComponentAccesses,
        CF: 'static + ComponentFilters,
        O: 'static,
    {
        {
            let mut ctymgr = self.galaxy.get_component_type_manager().write().unwrap();
            CA::maybe_register(&mut ctymgr);
            CF::maybe_register(&mut ctymgr);
        }
        self.raw_sys(SystemBuilder::create(
            std::any::type_name::<SystemFunction<CA, CF, O>>().to_string(),
            CA::accesses(),
            CF::filters(),
            mewo_ecs::SystemFunction(Box::new(
                move |galaxy, ev_insert, entity_transformer, access_key| {
                    (function)(SystemBus::create(
                        EntityBus::create(galaxy.get_component_type_manager(), entity_transformer),
                        EventBus::create(galaxy.get_event_manager(), ev_insert),
                        ResourceBus::create(galaxy.get_resource_manager()),
                        Components::create(
                            galaxy.get_component_type_manager(),
                            galaxy.get_archetype_manager(),
                            access_key,
                        ),
                    ));
                    true
                },
            )),
            None,
        ))
    }

    pub fn raw_sys(mut self, mut sys: SystemBuilder) -> Self {
        sys.set_plugin_name(self.raw_plugin.name.clone());
        self.raw_plugin.systems.push(sys);
        self
    }

    pub fn build(self) -> RawPlugin {
        self.raw_plugin
    }
}

pub trait Plugin {
    fn name() -> &'static str;
    fn plugin(pb: PluginBuilder) -> PluginBuilder;
    fn build_plugin(galaxy: &Galaxy) -> RawPlugin
    where
        Self: Sized,
    {
        Self::plugin(PluginBuilder::create::<Self>(galaxy)).build()
    }
}
