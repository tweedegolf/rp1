use casbin::prelude::Enforcer;
use casbin::CoreApi;
use casbin::{Result as CasbinResult, TryIntoAdapter, TryIntoModel};

use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use diesel::prelude::Expression;
use diesel::sql_types::Bool;

pub enum ReadListFilter {
    Filter(Box<dyn Expression<SqlType = Bool>>),
    Allow,
    Disallow,
}

pub trait CheckPermissions {
    type AuthUser;

    fn allow_read(&self, _: &Self::AuthUser) -> bool {
        true
    }

    fn allow_read_list(_: &Self::AuthUser) -> ReadListFilter {
        ReadListFilter::Allow
    }

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

#[derive(Debug)]
pub enum EnforcedBy<T> {
    Subject(T),
    SubjectAndDomain { subject: T, domain: String },
    ForbidAll,
}

impl<T> Default for EnforcedBy<T> {
    fn default() -> Self {
        EnforcedBy::ForbidAll
    }
}

#[derive(Clone, Debug)]
pub struct PermissionsGuard(Status);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PermissionsGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let status = request.local_cache(|| PermissionsGuard(Status::Forbidden));

        match *status {
            PermissionsGuard(status) if status == Status::Ok => {
                request::Outcome::Success(PermissionsGuard(Status::Ok))
            }
            PermissionsGuard(err_status) => request::Outcome::Failure((err_status, ())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NoPermissionsGuard;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for NoPermissionsGuard {
    type Error = ();

    async fn from_request(_: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        request::Outcome::Success(NoPermissionsGuard)
    }
}

type PermissionsEnforcer = std::sync::Arc<Enforcer>;

#[derive(Clone)]
pub struct PermissionsFairing<T> {
    pub enforcer: PermissionsEnforcer,
    pub marker: std::marker::PhantomData<T>,
}

impl<T> PermissionsFairing<T> {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(m: M, a: A) -> CasbinResult<Self> {
        let enforcer: Enforcer = Enforcer::new(m, a).await?;
        Ok(PermissionsFairing {
            enforcer: std::sync::Arc::new(enforcer),
            marker: std::marker::PhantomData,
        })
    }

    pub fn enforcer(&mut self) -> PermissionsEnforcer {
        self.enforcer.clone()
    }

    pub fn with_enforcer(e: PermissionsEnforcer) -> Self {
        PermissionsFairing {
            enforcer: e,
            marker: std::marker::PhantomData,
        }
    }
}

async fn casbin_enforce(
    enforcer: PermissionsEnforcer,
    enforce_arcs: impl casbin::EnforceArgs,
) -> PermissionsGuard {
    let guard = enforcer.enforce(enforce_arcs);

    let status = match guard {
        Ok(true) => Status::Ok,
        Ok(false) => Status::Forbidden,
        Err(_) => Status::BadGateway,
    };

    PermissionsGuard(status)
}

#[rocket::async_trait]
impl<T> Fairing for PermissionsFairing<T>
where
    T: Send + Sync + std::hash::Hash + serde::Serialize + 'static + std::fmt::Debug,
{
    fn info(&self) -> Info {
        Info {
            name: "PermissionsFairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let path = request.uri().path().to_string();
        let action = request.method().as_str().to_owned();
        let enforced_by: &EnforcedBy<T> = request.local_cache(EnforcedBy::default);

        let status = match enforced_by {
            EnforcedBy::Subject(subject) => {
                casbin_enforce(self.enforcer.clone(), (subject.to_owned(), path, action)).await
            }
            EnforcedBy::SubjectAndDomain { subject, domain } => {
                casbin_enforce(
                    self.enforcer.clone(),
                    (subject.to_owned(), domain.to_owned(), path, action),
                )
                .await
            }
            EnforcedBy::ForbidAll => PermissionsGuard(Status::Forbidden),
        };

        request.local_cache(|| status);
    }
}
