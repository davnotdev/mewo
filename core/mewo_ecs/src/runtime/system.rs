use super::galaxy::GalaxyRuntime;
use crate::{
    component::{
        ArchetypeAccessKey, ArchetypeManager, ComponentGroupQuery, ComponentHash,
        ComponentQueryAccessType, ComponentQueryFilterType, ComponentTypeId, ComponentTypeManager,
        EntityTransformer,
    },
    event::EventInsert,
    debug::prelude::*,
};

//  Returning true signals that bootstrapping / startup is complete.
//  For updates, return true. (This has no effect).
pub struct SystemFunction(
    pub  Box<
        dyn Fn(
            &GalaxyRuntime,
            &mut EventInsert,
            &mut EntityTransformer,
            ArchetypeAccessKey,
            // Option<ArchetypeAccess>,
            // usize,
            // usize,
        ) -> bool,
    >,
);

impl std::fmt::Debug for SystemFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SystemFunction")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EarlySystemPhase {
    Bootstrap,
    Startup,
}

#[derive(Debug)]
pub struct SystemBuilder {
    name: String,
    plugin_name: Option<String>,
    component_accesses: Vec<(ComponentHash, ComponentQueryAccessType)>,
    component_filters: Vec<(ComponentHash, ComponentQueryFilterType)>,
    function: SystemFunction,
    phase: Option<EarlySystemPhase>,
}

impl SystemBuilder {
    pub fn create(
        name: String,
        component_accesses: Vec<(ComponentHash, ComponentQueryAccessType)>,
        component_filters: Vec<(ComponentHash, ComponentQueryFilterType)>,
        function: SystemFunction,
        phase: Option<EarlySystemPhase>,
    ) -> SystemBuilder {
        SystemBuilder {
            name,
            plugin_name: None,
            component_accesses,
            component_filters,
            function,
            phase,
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
        let plugin_name = self.plugin_name.ok_or(InternalError {
            line: line!(),
            file: file!(),
            dumps: vec![DebugDumpTargets::Plugins],
            ty: InternalErrorType::SystemNoPluginName {
                system: self.name.clone(),
            },
            explain: Some(
                "
                A system by itself cannot know its plugin's name therefore, 
                it is the job of the burrito to set the system's plugin name 
                via `SystemBuilder::set_plugin_name`.",
            ),
        })?;
        Ok(System {
            name: self.name,
            plugin_name,
            function: self.function,
            archetype_access_key: amgr.create_archetype_access_key(query.clone())?,
            query,
            phase: self.phase,
        })
    }
}

pub struct System {
    pub name: String,
    pub plugin_name: String,
    pub query: ComponentGroupQuery,
    pub archetype_access_key: ArchetypeAccessKey,
    pub function: SystemFunction,
    pub phase: Option<EarlySystemPhase>,
}

impl System {
    pub fn run(
        &self,
        galaxy: &GalaxyRuntime,
        ev_insert: &mut EventInsert,
        transformer: &mut EntityTransformer,
    ) -> bool {
        // let akid = self.archetype_access_key;
        // let amgr = galaxy.get_archetype_manager();
        // let count = amgr.get_access_count(akid);
        // let mut res = true;
        // for idx in 0..count {
        //     loop {
        //         if let Some(access) = amgr.try_access(akid, idx).iex_unwrap() {
        //             res = res
        //                 && (self.function.0)(
        //                     galaxy,
        //                     ev_insert,
        //                     transformer,
        //                     Some(access),
        //                     idx,
        //                     count,
        //                 );
        //             break;
        //         }
        //         std::hint::spin_loop();
        //     }
        // }
        // if count == 0 {
        //     res = res && (self.function.0)(galaxy, ev_insert, transformer, None, 0, 1);
        // }
        (self.function.0)(galaxy, ev_insert, transformer, self.archetype_access_key)
    }
}
