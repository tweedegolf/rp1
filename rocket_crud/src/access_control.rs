use diesel::{BoxableExpression, backend::Backend, sql_types::Bool};

use crate::{CrudInsertable, CrudStruct, CrudUpdatable};

pub enum PermissionFilter<QS, DB> where DB: Backend {
    KeepAll,
    KeepNone,
    Filter(Box<dyn BoxableExpression<QS, DB, SqlType = Bool>>),
}

pub trait CheckPermissions where Self: CrudUpdatable + CrudInsertable + CrudStruct {
    type AuthUser;

    fn allow_read(&self, _: &Self::AuthUser) -> bool {
        true
    }

    fn filter_list<DB>(_: &Self::AuthUser) -> PermissionFilter<<Self as CrudStruct>::TableType, DB> where DB: Backend {
        PermissionFilter::KeepAll
    }

    fn allow_create(_new: &<Self as CrudInsertable>::InsertType, _: &Self::AuthUser) -> bool {
        true
    }

    fn allow_update(&self, _new: &<Self as CrudUpdatable>::UpdateType, _: &Self::AuthUser) -> bool {
        true
    }

    fn allow_delete(&self, _: &Self::AuthUser) -> bool {
        true
    }
}
