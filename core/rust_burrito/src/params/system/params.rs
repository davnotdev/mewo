use super::*;
use std::ops::Deref;

pub struct Events<E> {
    ev: Option<*const u8>,
    phantom: PhantomData<E>,
}

impl<E> Events<E>
where
    E: EventAccess,
{
    pub fn create(ev: Option<*const u8>) -> Self {
        Events {
            ev,
            phantom: PhantomData,
        }
    }
}

impl<E> Deref for Events<E>
where
    E: EventAccess,
{
    type Target = E;
    fn deref(&self) -> &Self::Target {
        E::data(&self.ev)
    }
}

pub struct Components<'sys, CA, CF> {
    ctymgr: &'sys ComponentTypeManager,
    access: &'sys Option<ArchetypeAccess<'sys>>,
    phantom: PhantomData<(CA, CF)>,
}

impl<'sys, CA, CF> Components<'sys, CA, CF>
where
    CA: ComponentAccesses,
    CF: ComponentFilters,
{
    pub fn create(
        ctymgr: &'sys ComponentTypeManager,
        access: &'sys Option<ArchetypeAccess<'sys>>,
    ) -> Self {
        Components {
            ctymgr,
            access,
            phantom: PhantomData,
        }
    }

    pub fn iter(&self) -> ComponentsIter<CA> {
        if let Some(access) = self.access {
            ComponentsIter::create(
                access.get_iter_count(),
                CA::hashes()
                    .into_iter()
                    .map(|hash| {
                        let cty = self.ctymgr.get_id_with_hash(hash).unwrap();
                        access.get_iter(cty).unwrap()
                    })
                    .collect(),
            )
        } else {
            ComponentsIter::create(0, vec![])
        }
    }

    pub fn eiter(&self) -> ComponentsEIter<CA> {
        if let a @ Some(access) = self.access {
            ComponentsEIter::create(
                access.get_iter_count(),
                CA::hashes()
                    .into_iter()
                    .map(|hash| {
                        let cty = self.ctymgr.get_id_with_hash(hash).unwrap();
                        access.get_iter(cty).unwrap()
                    })
                    .collect(),
                a,
            )
        } else {
            ComponentsEIter::create(0, vec![], &None)
        }
    }
}
