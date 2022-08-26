use super::*;

impl<E> EventAccess for E
where
    E: Event,
{
    fn hash() -> EventHash {
        E::event_trait_hash()
    }

    fn data(evmgr: &SharedEventManager) -> Option<&[Self]> {
        let evmgr = evmgr.read().unwrap();
        let ehash = E::event_trait_hash();
        if let Ok(ecount) = evmgr.get_event_count(ehash) {
            if ecount != 0 {
                return Some(unsafe {
                    std::slice::from_raw_parts(
                        evmgr.get_event(ehash, 0).unwrap() as *const E,
                        ecount,
                    )
                });
            }
        }
        None
    }
}
