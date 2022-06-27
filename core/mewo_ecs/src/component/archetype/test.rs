use super::super::{
    component_type::ComponentTypeEntry,
    transform::{EntityModifyBuilder, EntityTransformBuilder},
};
use super::*;
use crate::data::TVal;

//  TODO
//  Access is not tested here.
//  It's just too tedious to write :).

#[test]
fn test_archetype_transform() -> Result<()> {
    let mut ctymgr = ComponentTypeManager::create();

    let size_u8 = std::mem::size_of::<u8>();
    let size_f32 = std::mem::size_of::<f32>();
    let drop = |_| {};
    let clone = |ptr| TVal::create(0, ptr);

    let _comp_u8 = ctymgr.register(ComponentTypeEntry {
        name: "u8".to_string(),
        size: std::mem::size_of::<u8>(),
        hash: 0,
        drop,
        clone,
    })?;
    let _comp_f32 = ctymgr.register(ComponentTypeEntry {
        name: "f32".to_string(),
        size: std::mem::size_of::<f32>(),
        hash: 1,
        drop,
        clone,
    })?;

    let entity_a = Entity::from_id(10);
    let entity_b = Entity::from_id(24);

    let mut archetype = ArchetypeManager::create();
    archetype.insert_entity(entity_a)?;

    fn ptr<T>(v: T) -> *const u8 {
        &v as *const T as *const u8
    }

    let mut entity_transform =
        EntityTransformBuilder::create(EntityModifyBuilder::Modify(entity_a));
    entity_transform.insert(0, TVal::create(size_u8, ptr(10u8)));
    archetype.transform_entity(entity_transform.build(&ctymgr)?, &ctymgr)?;

    let mut entity_transform =
        EntityTransformBuilder::create(EntityModifyBuilder::Create(Some(entity_b)));
    entity_transform.insert(0, TVal::create(size_u8, ptr(12u8)));
    entity_transform.insert(1, TVal::create(size_f32, ptr(8.5f32)));
    archetype.transform_entity(entity_transform.build(&ctymgr)?, &ctymgr)?;

    let entity_transform = EntityTransformBuilder::create(EntityModifyBuilder::Destroy(entity_b));
    archetype.transform_entity(entity_transform.build(&ctymgr)?, &ctymgr)?;
    let entity_transform = EntityTransformBuilder::create(EntityModifyBuilder::Destroy(entity_b));
    assert!(archetype
        .transform_entity(entity_transform.build(&ctymgr)?, &ctymgr)
        .is_err());

    Ok(())
}
