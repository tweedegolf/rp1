
use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use diesel::prelude::Expression;
use diesel::sql_types::Bool;

// pub enum ReadListFilter {
//     Filter(Box<dyn Expression<SqlType = Bool>>),
//     Allow,
//     Disallow,
// }

pub trait CheckPermissions {
    type AuthUser;

    fn allow_read(&self, _: &Self::AuthUser) -> bool {
        true
    }

    // fn allow_read_list(_: &Self::AuthUser) -> ReadListFilter {
    //     ReadListFilter::Allow
    // }

    fn allow_create(&self, _: &Self::AuthUser) -> bool {
        true
    }

    fn allow_update(&self, _new: &Self, _: &Self::AuthUser) -> bool {
        true
    }

    fn allow_delete(&self, _: &Self::AuthUser) -> bool {
        true
    }
}
