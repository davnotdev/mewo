use super::*;

pub trait ComponentAccessesNormal {
    fn hashes() -> Vec<ComponentTypeId>;
    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>);
}

impl<C0> ComponentAccessesNormal for C0
where
    C0: GenericComponent + 'static,
{
    fn hashes() -> Vec<ComponentTypeId> {
        vec![C0::mewo_component_id()]
    }

    fn component_maybe_insert(ctyp: &RwLock<ComponentTypePlanet>) {
        component_maybe_insert::<C0>(ctyp)
    }
}
