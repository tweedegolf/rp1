use casbin::prelude::Enforcer;
use casbin::CoreApi;
use casbin::{Result as CasbinResult, TryIntoAdapter, TryIntoModel};

use rocket::data::Data;
use rocket::fairing::{Fairing, Info, Kind};
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};

use async_mutex::Mutex;

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

#[derive(Clone)]
pub struct PermissionsGuard(Status);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for PermissionsGuard {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        match *request.local_cache(|| PermissionsGuard(Status::BadGateway)) {
            PermissionsGuard(status) if status == Status::Ok => {
                request::Outcome::Success(PermissionsGuard(Status::Ok))
            }
            PermissionsGuard(err_status) => request::Outcome::Failure((err_status, ())),
        }
    }
}

type PermissionsEnforcer = std::sync::Arc<Mutex<Enforcer>>;

#[derive(Clone)]
pub struct PermissionsFairing {
    pub enforcer: PermissionsEnforcer,
}

impl PermissionsFairing {
    pub async fn new<M: TryIntoModel, A: TryIntoAdapter>(m: M, a: A) -> CasbinResult<Self> {
        let enforcer: Enforcer = Enforcer::new(m, a).await?;
        Ok(PermissionsFairing {
            enforcer: std::sync::Arc::new(Mutex::new(enforcer)),
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
    let mut mutex_guard = enforcer.lock().await;
    let guard = mutex_guard.enforce_mut(enforce_arcs);
    drop(mutex_guard);

    PermissionsGuard(match guard {
        Ok(true) => Status::Ok,
        Ok(false) => Status::Forbidden,
        Err(_) => Status::BadGateway,
    })
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
        match enforced_by {
            EnforcedBy::Subject(subject) => {
                let status = casbin_enforce(
                    self.enforcer.clone(),
                    vec![subject.to_owned(), path, action],
                )
                .await;
                request.local_cache(|| status);
            }
            EnforcedBy::SubjectAndDomain { subject, domain } => {
                let status = casbin_enforce(
                    self.enforcer.clone(),
                    vec![subject.to_owned(), domain.to_owned(), path, action],
                )
                .await;
                request.local_cache(|| status);
            }
            EnforcedBy::ForbidAll => {
                request.local_cache(|| PermissionsGuard(Status::BadGateway));
            }
        }
    }
}
