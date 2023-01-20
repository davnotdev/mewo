use super::*;

//      DVec    DVec    DVec
//  e   d       d       d
//  e   d       d       d
#[derive(Debug)]
pub(super) struct StorageBloc {
    datas: Vec<(ComponentTypeId, StorageRow)>,
    entities: Vec<Entity>,
}

impl StorageBloc {
    pub fn new(planet: &ComponentTypePlanet, group: &ComponentGroup) -> Result<Self> {
        Ok(StorageBloc {
            datas: group
                .get_components()
                .iter()
                .map(|&cty| {
                    let ty = planet.get_type(cty)?;
                    Ok((
                        cty,
                        match ty.dup {
                            ValueDuplicate::None | ValueDuplicate::Clone(_) => {
                                StorageRow::Normal(RwLock::new(DVec::new(ty.size, ty.drop)))
                            }
                            ValueDuplicate::Copy => StorageRow::CopyCat(
                                Mutex::new(DVec::new(ty.size, ty.drop)),
                                DVec::new(ty.size, ty.drop),
                            ),
                        },
                    ))
                })
                .collect::<Result<_>>()?,
            entities: Vec::new(),
        })
    }

    pub fn get_len(&self) -> usize {
        if let Some((_, row)) = self.datas.get(0) {
            row.get_len()
        } else {
            //  Null Storage
            self.entities.len()
        }
    }

    pub fn get_write_lock(&self, id: ComponentTypeId) -> Option<()> {
        self.datas
            .get(self.type_column(id)?)
            .unwrap()
            .1
            .write_lock();
        Some(())
    }

    pub fn get_write_unlock(&self, id: ComponentTypeId) -> Option<()> {
        self.datas
            .get(self.type_column(id)?)
            .unwrap()
            .1
            .write_unlock();
        Some(())
    }

    pub fn get_read_lock(&self, id: ComponentTypeId) -> Option<()> {
        self.datas.get(self.type_column(id)?).unwrap().1.read_lock();
        Some(())
    }

    pub fn get_read_unlock(&self, id: ComponentTypeId) -> Option<()> {
        self.datas
            .get(self.type_column(id)?)
            .unwrap()
            .1
            .read_unlock();
        Some(())
    }

    pub fn get_write(&self, id: ComponentTypeId) -> Option<*const u8> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .access_write(),
        )
    }

    pub fn get_read(&self, id: ComponentTypeId) -> Option<*const u8> {
        Some(
            self.datas
                .get(self.type_column(id)?)
                .unwrap()
                .1
                .access_read(),
        )
    }

    pub fn get_entities(&self) -> *const Entity {
        self.entities.as_ptr()
    }

    pub fn insert_entity(&mut self, entity: Entity, ins: StorageBlocInsert) -> Result<()> {
        assert!(ins.components.len() == self.datas.len());
        //  TODO FIX: Should never panic. It's here right now just in case.
        assert!(self.entity_row(entity).is_none());
        self.entities.push(entity);
        for (id, val) in ins.components.into_iter() {
            let column = self.type_column(id).ok_or_else(|| {
                ecs_err!(ErrorType::StorageBlocInsertComponent { id, entity }, self)
            })?;
            self.datas.get_mut(column).unwrap().1.resize(1, val);
        }
        Ok(())
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        let row = self
            .entity_row(entity)
            .ok_or(ecs_err!(ErrorType::StorageBlocRemove { entity }, self))?;
        self.entities.remove(row);
        for data in self.datas.iter_mut() {
            data.1.swap_remove(row);
        }
        Ok(())
    }

    //  Don't drop components.
    pub fn take_remove_entity(&mut self, entity: Entity) -> Result<()> {
        let row = self
            .entity_row(entity)
            .ok_or(ecs_err!(ErrorType::StorageBlocRemove { entity }, self))?;
        self.entities.remove(row);
        for data in self.datas.iter_mut() {
            data.1.take_swap_remove(row);
        }
        Ok(())
    }

    pub(super) fn copy_entity(
        src: &mut Self,
        dst: &mut Self,
        entity: Entity,
        mut missings: StorageBlocInsert,
    ) -> Result<()> {
        let src_row = src.entity_row(entity).ok_or(ecs_err!(
            ErrorType::StorageBlocCopyEntity { entity },
            (&src, &dst)
        ))?;
        for (id, data) in src.datas.iter_mut() {
            missings.insert(*id, data.get_mut(src_row).unwrap());
        }
        dst.insert_entity(entity, missings)?;
        src.take_remove_entity(entity)?;

        Ok(())
    }

    pub fn get_entity_idx(&self, entity: Entity) -> Option<usize> {
        self.entity_row(entity)
    }

    pub fn update(&mut self) {
        for (_, row) in self.datas.iter_mut() {
            row.update();
        }
    }
}

impl StorageBloc {
    fn type_column(&self, cty: ComponentTypeId) -> Option<Column> {
        self.datas.iter().position(|&(p, _)| p == cty)
    }

    fn entity_row(&self, e: Entity) -> Option<Row> {
        self.entities.iter().position(|&p| p == e)
    }
}

pub(super) struct StorageBlocInsert {
    components: HashMap<ComponentTypeId, *const u8>,
}

impl StorageBlocInsert {
    pub fn new() -> Self {
        StorageBlocInsert {
            components: HashMap::new(),
        }
    }

    //  A double insert results in one value being replaced.
    //  Do we want this behavior?
    pub fn insert(&mut self, id: ComponentTypeId, val: *const u8) {
        self.components.insert(id, val);
    }
}
