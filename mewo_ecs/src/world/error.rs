use super::entity::Entity;

#[derive(Debug, PartialEq)]
pub enum ECSError {
    ComponentTypeExists(&'static str),
    ComponentTypeDoesNotExist,
    EntityAlreadyHasComponent(Entity, &'static str),
    EntityDoesNotHaveComponent(Entity, &'static str),
    EntityDoesNotExist(Entity),
}

pub type Result<T> = std::result::Result<T, ECSError>;
