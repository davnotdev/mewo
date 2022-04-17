use super::component::ComponentTypeId;
use super::world::World;
use super::mask::BoolMask;

pub struct ComponentStamp {
    mask: BoolMask,
}

impl ComponentStamp {
    pub fn create(scene: &World) -> ComponentStamp {
        let mut mask = BoolMask::create();
        let len = scene
            .get_component_manager()
            .get_component_type_count();
        mask.extend(len-mask.get_len());
        ComponentStamp { 
            mask, 
        }
    }

    pub fn from(mask: BoolMask) -> ComponentStamp {
        ComponentStamp { 
            mask,
        }
    }
    pub fn stamp(&mut self, id: ComponentTypeId) -> &mut Self {
        match self.mask.set(id, true) {
            Err(_) => { unreachable!("Only reachable if self.mask.extend fails") }
            _ => {}
        }
        self
    }

    pub fn unstamp(&mut self, id: ComponentTypeId) -> &mut Self {
        match self.mask.set(id, false) {
            Err(_) => { unreachable!("Only reachable if self.mask.extend fails") }
            _ => {}
        }
        self
    }

    pub fn get_mask(&self) -> &BoolMask {
        &self.mask
    }

    pub fn get_mut_mask(&mut self) -> &mut BoolMask {
        &mut self.mask
    }

    pub fn get(&self, i: usize) -> bool {
        match self.mask.get(i) {
            Ok(res) => res,
            _ => panic!("The Component Id `{}` does not exist", i)
        }
    }

    pub fn set(&mut self, i: usize, val: bool) {
        match self.mask.set(i, val) {
            Ok(res) => res,
            _ => panic!("The Component Id `{}` does not exist", i)
        }
    }

    pub fn is_empty(&self) -> bool {
        self.mask.is_empty()
    }

    pub fn compare(&self, other: &ComponentStamp) -> bool {
        match self.mask.compare(&other.mask) {
            Ok(res) => res,
            _ => unreachable!("Only reachable if component stamps have different lengths")
        }
    }

    pub fn bitwise_and(&self, other: &ComponentStamp) -> ComponentStamp {
        match self.mask.bitwise_and(&other.mask) {
            Ok(res) => ComponentStamp::from(res),
            _ => unreachable!("Only reachable if component stamps have different lengths")
        }
    }
}

