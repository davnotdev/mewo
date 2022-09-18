use super::{Component, ComponentTypeId, ComponentTypePlanet, QueryAccessType};
use parking_lot::RwLock;

//  TODO EXT: Use macros!

//  Used by filter.
mod normal;

//  Used by query.
mod optional;

//  Used by entity get.
mod nonoptional;

pub use nonoptional::{ComponentAccessNonOptional, ComponentAccessesNonOptional};
pub use normal::ComponentAccessesNormal;
pub use optional::{ComponentAccessOptional, ComponentAccessesOptional};

fn component_maybe_insert<C: Component + 'static>(ctyp: &RwLock<ComponentTypePlanet>) {
    let id = C::mewo_component_id();
    if ctyp.read().get_type(id).is_err() {
        ctyp.write()
            .insert_type(id, C::mewo_component_info())
            .unwrap();
    }
}
