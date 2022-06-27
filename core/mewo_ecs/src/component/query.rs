use super::ComponentTypeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentQueryAccessType {
    Read,
    Write,
    OptionRead,
    OptionWrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentQueryFilterType {
    With,
    Without,
}

#[derive(Debug, Clone)]
pub struct ComponentGroupQuery {
    accesses: Vec<(ComponentTypeId, ComponentQueryAccessType)>,
    filters: Vec<(ComponentTypeId, ComponentQueryFilterType)>,
}

impl ComponentGroupQuery {
    pub fn create() -> Self {
        ComponentGroupQuery {
            accesses: Vec::new(),
            filters: Vec::new(),
        }
    }

    pub fn add_access(&mut self, cty: ComponentTypeId, aty: ComponentQueryAccessType) {
        self.accesses.push((cty, aty));
    }

    pub fn add_filter(&mut self, cty: ComponentTypeId, fty: ComponentQueryFilterType) {
        self.filters.push((cty, fty));
    }

    pub fn get_accesses(&self) -> &Vec<(ComponentTypeId, ComponentQueryAccessType)> {
        &self.accesses
    }

    pub fn get_filters(&self) -> &Vec<(ComponentTypeId, ComponentQueryFilterType)> {
        &self.filters
    }
}
