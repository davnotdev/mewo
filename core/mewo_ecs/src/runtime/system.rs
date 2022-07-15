use super::galaxy::GalaxyRuntime;
use crate::{
    component::{
        ArchetypeAccess, ArchetypeAccessKey, ArchetypeManager, ComponentGroupQuery, ComponentHash,
        ComponentQueryAccessType, ComponentQueryFilterType, ComponentTypeId, ComponentTypeManager,
        EntityTransformer,
    },
    error::*,
    event::{EventHash, EventInsert, EventOption},
    resource::ResourceManager,
};

type SystemFunction = Box<
    dyn Fn(
        &GalaxyRuntime,
        Option<*const u8>,
        &ResourceManager,
        &mut EventInsert,
        &mut EntityTransformer,
        Option<ArchetypeAccess>,
        usize,
        usize,
    ),
>;

pub struct SystemBuilder {
    name: String,
    plugin_name: Option<String>,
    component_accesses: Vec<(ComponentHash, ComponentQueryAccessType)>,
    component_filters: Vec<(ComponentHash, ComponentQueryFilterType)>,
    event: EventOption<EventHash>,
    function: SystemFunction,
}

impl SystemBuilder {
    pub fn create(
        name: String,
        event: EventOption<EventHash>,
        component_accesses: Vec<(ComponentHash, ComponentQueryAccessType)>,
        component_filters: Vec<(ComponentHash, ComponentQueryFilterType)>,
        function: SystemFunction,
    ) -> SystemBuilder {
        SystemBuilder {
            name,
            plugin_name: None,
            component_accesses,
            component_filters,
            event,
            function,
        }
    }

    pub fn set_plugin_name(&mut self, plugin_name: String) {
        self.plugin_name = Some(plugin_name);
    }

    pub fn build(
        self,
        ctymgr: &ComponentTypeManager,
        amgr: &mut ArchetypeManager,
    ) -> Result<System> {
        let accesses: Vec<(Result<ComponentTypeId>, ComponentQueryAccessType)> = self
            .component_accesses
            .into_iter()
            .map(|(hash, qt)| (ctymgr.get_id_with_hash(hash), qt))
            .collect();
        let filters: Vec<(Result<ComponentTypeId>, ComponentQueryFilterType)> = self
            .component_filters
            .into_iter()
            .map(|(hash, qt)| (ctymgr.get_id_with_hash(hash), qt))
            .collect();
        let mut query = ComponentGroupQuery::create();

        //  Would be nice to have this be apart of the above chains.
        for (cty, qt) in accesses {
            let cty = cty?;
            query.add_access(cty, qt)
        }
        for (cty, qt) in filters {
            let cty = cty?;
            query.add_filter(cty, qt)
        }
        let plugin_name = self.plugin_name.ok_or(RuntimeError::SystemNoPluginName {
            system: self.name.clone(),
        })?;
        Ok(System {
            name: self.name,
            plugin_name,
            function: self.function,
            archetype_access_key: amgr.create_archetype_access_key(query.clone())?,
            event: self.event,
            query,
        })
    }
}

pub struct System {
    pub name: String,
    pub plugin_name: String,
    pub query: ComponentGroupQuery,
    pub archetype_access_key: ArchetypeAccessKey,
    pub event: EventOption<EventHash>,
    pub function: SystemFunction,
}

impl System {
    pub fn run(
        &self,
        galaxy: &GalaxyRuntime,
        ev: Option<*const u8>,
        rcmgr: &ResourceManager,
        ev_insert: &mut EventInsert,
        transformer: &mut EntityTransformer,
    ) {
        let akid = self.archetype_access_key;
        let amgr = galaxy.get_archetype_manager();
        let count = amgr.get_access_count(akid);
        for idx in 0..count {
            loop {
                if let Some(access) = amgr.try_access(akid, idx).unwrap() {
                    (self.function)(
                        galaxy,
                        ev,
                        rcmgr,
                        ev_insert,
                        transformer,
                        Some(access),
                        idx,
                        count,
                    );
                    break;
                }
                std::hint::spin_loop();
            }
        }
        if count == 0 {
            (self.function)(galaxy, ev, rcmgr, ev_insert, transformer, None, 0, 1);
        }
    }
}
