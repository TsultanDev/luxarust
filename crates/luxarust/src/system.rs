use std::any::TypeId;

use crate::app::App;

pub(crate) type SystemFn = Box<dyn FnMut(&mut App)>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SystemId(pub(crate) u64);

pub(crate) struct SystemSlot {
    pub(crate) id: SystemId,
    pub(crate) owner: Option<TypeId>,
    pub(crate) func: SystemFn,
}