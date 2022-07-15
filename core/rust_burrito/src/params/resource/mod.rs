use super::super::resource::Resource;
use mewo_ecs::{ResourceHash, ResourceManager, ResourceQueryAccessType};
mod impls;

pub trait Resources {
    fn accesses() -> Vec<(ResourceHash, ResourceQueryAccessType)>;
    fn lock(rcmgr: &ResourceManager);
    fn unlock(rcmgr: &ResourceManager);
    fn get(rcmgr: &ResourceManager) -> Option<Self>
    where
        Self: Sized;
}

trait ResourceAccess {
    fn data(data: *const u8) -> Self;
    fn hash() -> ResourceHash;
    fn access() -> ResourceQueryAccessType;
}
