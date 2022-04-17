use std::marker::PhantomData;
use std::any::TypeId;
use super::system::ComponentAccessMode;
use super::component::Component;

pub struct WishType {
    pub tyid: TypeId,
    pub with: Option<Vec<TypeId>>,
    pub without: Option<Vec<TypeId>>,
    pub access_mode: ComponentAccessMode,
}

//  (Write<...>, Read<...>)
pub trait WishArg {
    fn get_types() -> Vec<WishType>;
}

pub trait WishArgType {
    fn get_type() -> WishType;
}

pub trait QueryTypeArgFilter {
    fn get_filter_types() -> Option<Vec<TypeId>>;
}

pub struct Read<C, With=(), Without=()> 
    where C: Component
{
    c: PhantomData<C>,
    with: PhantomData<With>,
    without: PhantomData<Without>,
}
pub struct Write<C, With=(), Without=()> 
    where C: Component
{
    c: PhantomData<C>,
    with: PhantomData<With>,
    without: PhantomData<Without>,
}

impl<C, With, Without> WishArgType for Read<C, With, Without> 
    where 
    C: 'static + Component,
    With: QueryTypeArgFilter,
    Without: QueryTypeArgFilter,
{
    fn get_type() -> WishType {
        WishType {
            tyid: TypeId::of::<C>(),
            with: With::get_filter_types(),
            without: Without::get_filter_types(),
            access_mode: ComponentAccessMode::Read,
        }
    }
}
impl<C, With, Without> WishArgType for Write<C, With, Without> 
    where 
    C: 'static + Component,
    With: QueryTypeArgFilter,
    Without: QueryTypeArgFilter,
{
    fn get_type() -> WishType {
        WishType {
            tyid: TypeId::of::<C>(),
            with: With::get_filter_types(),
            without: Without::get_filter_types(),
            access_mode: ComponentAccessMode::Write,
        }
    }
}

//  TODO: use macros silly!

impl WishArg for () {
    fn get_types() -> Vec<WishType> { Vec::new()
    }
}

impl<C0> WishArg for C0 
    where 
    C0: WishArgType,
{
    fn get_types() -> Vec<WishType> {
        vec![
            C0::get_type()
        ]
    }
}

impl<C0, C1> WishArg for (C0, C1) 
    where 
    C0: WishArgType, 
    C1: WishArgType,
{
    fn get_types() -> Vec<WishType> {
        vec![
            C0::get_type(),
            C1::get_type(),
        ]
    }
}

impl<C0, C1, C2> WishArg for (C0, C1, C2) 
    where 
    C0: WishArgType,
    C1: WishArgType,
    C2: WishArgType,
{
    fn get_types() -> Vec<WishType> {
        vec![
            C0::get_type(),
            C1::get_type(),
            C2::get_type(),
        ]
    }
}

impl QueryTypeArgFilter for () {
    fn get_filter_types() -> Option<Vec<TypeId>> {
        None
    }
}

impl<F0> QueryTypeArgFilter for F0
    where
    F0: 'static + Component,
{
    fn get_filter_types() -> Option<Vec<TypeId>> {
        Some(vec![
            TypeId::of::<F0>(),
        ])
    }
}

impl<F0, F1> QueryTypeArgFilter for (F0, F1) 
    where 
    F0: 'static + Component,
    F1: 'static + Component
{
    fn get_filter_types() -> Option<Vec<TypeId>> {
        Some(vec![
            TypeId::of::<F0>(),
            TypeId::of::<F1>(),
        ])
    }
}

impl<F0, F1, F2> QueryTypeArgFilter for (F0, F1, F2) 
    where 
    F0: 'static + Component,
    F1: 'static + Component,
    F2: 'static + Component,
{
    fn get_filter_types() -> Option<Vec<TypeId>> {
        Some(vec![
            TypeId::of::<F0>(),
            TypeId::of::<F1>(),
            TypeId::of::<F2>(),
        ])
    }
}
