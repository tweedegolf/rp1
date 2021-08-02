use diesel::{BoxableExpression, backend::Backend, query_builder::BoxedSelectStatement, sql_types::Bool};

use crate::{CrudInsertable, CrudStruct, CrudUpdatable};

/// A filter for when a list of items that will be queried needs to be
pub enum PermissionFilter<QS, DB> where DB: Backend {
    KeepAll,
    KeepNone,
    Filter(Box<dyn BoxableExpression<QS, DB, SqlType = Bool>>),
}

impl<'a, QS, DB> PermissionFilter<QS, DB> where DB: 'a + Backend, QS: 'a {
    pub fn apply<ST>(self, query: BoxedSelectStatement<'a, ST, QS, DB>) -> Option<BoxedSelectStatement<'a, ST, QS, DB>> {
        use diesel::prelude::*;
        match self {
            PermissionFilter::KeepAll => Some(query),
            PermissionFilter::KeepNone => None,
            PermissionFilter::Filter(f) => {
                Some(query.filter(f))
            }
        }
    }
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
