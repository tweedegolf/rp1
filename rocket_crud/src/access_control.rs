use casbin::prelude::Enforcer;
use casbin::CoreApi;
use casbin::{Result as CasbinResult, TryIntoAdapter, TryIntoModel};

use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

#[derive(Debug)]
pub enum EnforcedBy {
    Subject(String),
    SubjectAndDomain { subject: String, domain: String },
    ForbidAll,
}

impl Default for EnforcedBy {
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
        dbg!(&status);

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
pub struct PermissionsFairing {
    pub enforcer: PermissionsEnforcer,
}

impl PermissionsFairing {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(m: M, a: A) -> CasbinResult<Self> {
        let enforcer: Enforcer = Enforcer::new(m, a).await?;
        Ok(PermissionsFairing {
            enforcer: std::sync::Arc::new(enforcer),
        })
    }

    pub fn enforcer(&mut self) -> PermissionsEnforcer {
        self.enforcer.clone()
    }

    pub fn with_enforcer(e: PermissionsEnforcer) -> PermissionsFairing {
        PermissionsFairing { enforcer: e }
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
impl Fairing for PermissionsFairing {
    fn info(&self) -> Info {
        Info {
            name: "PermissionsFairing",
            kind: Kind::Request | Kind::Response,
        }
    }

    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        let path = request.uri().path().to_string();
        let action = request.method().as_str().to_owned();

        let enforced_by = request.local_cache(EnforcedBy::default);

        let status = match enforced_by {
            EnforcedBy::Subject(subject) => {
                dbg!(&subject, &path, &action);
                casbin_enforce(
                    self.enforcer.clone(),
                    vec![subject.to_owned(), path, action],
                )
                .await
            }
            EnforcedBy::SubjectAndDomain { subject, domain } => {
                casbin_enforce(
                    self.enforcer.clone(),
                    vec![subject.to_owned(), domain.to_owned(), path, action],
                )
                .await
            }
            EnforcedBy::ForbidAll => PermissionsGuard(Status::Forbidden),
        };

        request.local_cache(|| status);
    }
}
