use super::component_manager::ComponentTypeId;
use super::component_stamp::ComponentStamp;
use super::command::WorldCommands;
use super::gift::GiftInstance;
use super::world::World;

pub type MainSystem = fn (&mut GiftInstance, &mut WorldCommands) -> ();

pub struct PickyGiftFilter {
    pub with: Option<ComponentStamp>,
    pub without: Option<ComponentStamp>,
}

pub struct SantaClaus {
    pub writes: Vec<(ComponentTypeId, PickyGiftFilter)>,
    pub reads: Vec<(ComponentTypeId, PickyGiftFilter)>,
}

impl SantaClaus {
    pub fn wishlist<'world>(scene: &'world World) -> SantaClausWishList<'world> {
        SantaClausWishList {
            world: scene,
            writes_data: Vec::new(),
            reads_data: Vec::new(),
        } 
    }
}

pub struct SantaClausWishList<'world> {
    world: &'world World,
    writes_data: Vec<(ComponentTypeId, PickyGiftFilter)>,
    reads_data: Vec<(ComponentTypeId, PickyGiftFilter)>,
}

impl<'world> SantaClausWishList<'world> {
    pub fn writes(
        mut self, 
        writes: Vec<ComponentTypeId>,
        with: Option<Vec<ComponentTypeId>>,
        without: Option<Vec<ComponentTypeId>>,
    ) -> Self {
        for id in writes.into_iter() {
            self.writes_data.push((
                id,
                PickyGiftFilter {
                    with: if let Some(with) = &with {
                        let mut stamp = ComponentStamp::create(self.world);
                        with.iter().for_each(|id| { stamp.stamp(*id); });
                        Some(stamp)
                    } else { None },
                    without: if let Some(without) = &without {
                        let mut stamp = ComponentStamp::create(self.world);
                        without.iter().for_each(|id| { stamp.stamp(*id); });
                        Some(stamp)
                    } else { None },
                },
            ))
        }
        self
    }

    pub fn reads(
        mut self, 
        reads: Vec<ComponentTypeId>,
        with: Option<Vec<ComponentTypeId>>,
        without: Option<Vec<ComponentTypeId>>,
    ) -> Self {
        for id in reads.into_iter() {
            self.reads_data.push((
                id,
                PickyGiftFilter {
                    with: if let Some(with) = &with {
                        let mut stamp = ComponentStamp::create(self.world);
                        with.iter().for_each(|id| { stamp.stamp(*id); });
                        Some(stamp)
                    } else { None },
                    without: if let Some(without) = &without {
                        let mut stamp = ComponentStamp::create(self.world);
                        without.iter().for_each(|id| { stamp.stamp(*id); });
                        Some(stamp)
                    } else { None },
                },
            ))
        }
        self 
    }

    pub fn finish(self) -> SantaClaus {
        SantaClaus {
            writes: self.writes_data,
            reads: self.reads_data,
        }
    }
}

