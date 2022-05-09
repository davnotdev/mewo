use crate::{
    Entity,
    ComponentTypeId,
};

#[derive(Debug, PartialEq)]
pub enum ComponentErrorIdentifier {
    Unknown,
    Name(&'static str),
    Id(ComponentTypeId),
}

#[derive(Debug, PartialEq)]
pub enum ECSError {
    ComponentTypeExists(ComponentErrorIdentifier),
    ComponentTypeDoesNotExist(ComponentErrorIdentifier),
    EntityAlreadyHasComponent(Entity, ComponentErrorIdentifier),
    EntityDoesNotHaveComponent(Entity, ComponentErrorIdentifier),
    EntityDoesNotExist(Entity),
    ResourceNotFound(&'static str),
}

pub type Result<T> = std::result::Result<T, ECSError>;
