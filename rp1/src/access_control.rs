//! Authorization and access control support.

use diesel::{
    backend::Backend, query_builder::BoxedSelectStatement, sql_types::Bool, BoxableExpression,
};

use crate::{CrudInsertable, CrudStruct, CrudUpdatable};

/// A filter for when a list of items that will be queried needs to be filtered.
pub enum PermissionFilter<QS, DB>
where
    DB: Backend,
{
    /// Keep all rows, and do not exclude any row in the results.
    KeepAll,
    /// Disallow reading of any row from the database.
    KeepNone,
    /// Add a filter to the query that excludes a part of the results.
    Filter(Box<dyn BoxableExpression<QS, DB, SqlType = Bool>>),
}

impl<'a, QS, DB> PermissionFilter<QS, DB>
where
    DB: 'a + Backend,
    QS: 'a,
{
    pub fn apply<ST>(
        self,
        query: BoxedSelectStatement<'a, ST, QS, DB>,
    ) -> Option<BoxedSelectStatement<'a, ST, QS, DB>> {
        use diesel::prelude::*;
        match self {
            PermissionFilter::KeepAll => Some(query),
            PermissionFilter::KeepNone => None,
            PermissionFilter::Filter(f) => Some(query.filter(f)),
        }
    }
}

/// Trait that needs to be implemented to support authorization.
pub trait CheckPermissions
where
    Self: CrudUpdatable + CrudInsertable + CrudStruct,
{
    /// This associated type will be added as a guard to all routes. One of the
    /// specific filter functions will then be called based on what route needs
    /// a permission check.
    type AuthUser;

    /// This function should return a boolean indicating if the user has access
    /// to read a single item (`self`). By default all reads are allowed.
    fn allow_read(&self, _: &Self::AuthUser) -> bool {
        true
    }

    /// This function should return a [PermissionFilter] that indicates what
    /// rows to filter from the database.
    fn filter_list<DB>(_: &Self::AuthUser) -> PermissionFilter<<Self as CrudStruct>::TableType, DB>
    where
        DB: Backend,
    {
        PermissionFilter::KeepAll
    }

    /// This function should return a boolean indicating if the user has access
    /// to create the new item. By default all creates are allowed.
    fn allow_create(_new: &<Self as CrudInsertable>::InsertType, _: &Self::AuthUser) -> bool {
        true
    }

    /// This function should return a boolean indicating if the user has access
    /// to update the existing item (`self`) to the new given item. By default
    /// all updates are allowed.
    fn allow_update(&self, _new: &<Self as CrudUpdatable>::PutType, _: &Self::AuthUser) -> bool {
        true
    }

    /// This function should return a boolean indicating if the user has access
    /// to delete the existing item (`self`). By default all deletes are
    /// allowed.
    fn allow_delete(&self, _: &Self::AuthUser) -> bool {
        true
    }
}
