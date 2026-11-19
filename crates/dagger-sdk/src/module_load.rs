//! Once-per-session loading of the modules that generated module clients are bound to.
//!
//! A generated module client's requests are only valid once the engine serves the
//! client's module to the session. Each session keeps one entry for every
//! `(address, pin)` it was asked for: the first request drives the load, concurrent
//! requests wait for that load's outcome, and a failed or abandoned load leaves no
//! entry, so the next request starts again. This is the bookkeeping of `ensureModule`
//! in Dagger's Go SDK, with one addition for Rust's cancellation: the load runs inside
//! the future of the request that started it, so dropping that future abandons the
//! load and one of the waiting requests takes it over.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, PoisonError};

use tokio::sync::watch;

use crate::errors::{ModuleClientError, ModuleClientErrorKind, QueryError, RequestError};
use crate::graphql::{GraphQlError, RawResponse};

/// The module a generated client's requests need the engine to serve first.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct ModuleRequirement {
    address: String,
    pin: String,
}

impl ModuleRequirement {
    pub(crate) fn new(address: &str, pin: &str) -> Self {
        Self {
            address: address.to_owned(),
            pin: pin.to_owned(),
        }
    }

    /// The module's address: a git ref, or a local module's path in its workspace.
    pub(crate) fn address(&self) -> &str {
        &self.address
    }

    /// The commit a git address is pinned to, empty for a local module.
    pub(crate) fn pin(&self) -> &str {
        &self.pin
    }

    /// Names the engine's refusal to serve the module.
    ///
    /// Only an engine without `serveModule` is told apart: every other failure, the
    /// module's own included, is a failed load of this address.
    pub(crate) fn load_error(&self, cause: QueryError) -> QueryError {
        let kind = if is_missing_field(&cause) {
            ModuleClientErrorKind::Unsupported
        } else {
            ModuleClientErrorKind::Load
        };
        QueryError::ModuleClient(ModuleClientError::new(kind, &self.address, cause))
    }

    /// Names a request's failure when it shows the client no longer matches the module.
    ///
    /// The module is loaded by then, so a field the schema lacks is one the client was
    /// generated with and the module has since lost. Any other failure is returned
    /// unchanged.
    pub(crate) fn request_error(&self, error: QueryError) -> QueryError {
        if is_missing_field(&error) {
            return QueryError::ModuleClient(ModuleClientError::new(
                ModuleClientErrorKind::OutOfDate,
                &self.address,
                error,
            ));
        }
        error
    }
}

/// Whether `error` carries the engine's validation error for a field the schema lacks.
///
/// The engine reports it in the response's error list, or in the body of an
/// unsuccessful HTTP response.
fn is_missing_field(error: &QueryError) -> bool {
    let response: Option<&RawResponse> = match error {
        QueryError::GraphQl { response } | QueryError::Exec { response, .. } => Some(response),
        QueryError::Request(RequestError::Connection(connection)) => connection.raw_response(),
        _ => None,
    };
    response.is_some_and(|response| response.errors().iter().any(is_missing_field_error))
}

/// The validation error for an unknown field, as `missingFieldError` in Dagger's Go SDK
/// recognises it. A resolver that quotes the same message carries a path, and other
/// validation failures carry another message, so neither matches.
fn is_missing_field_error(error: &GraphQlError) -> bool {
    error.path().is_empty()
        && error
            .extensions()
            .and_then(|extensions| extensions.get("code"))
            .and_then(serde_json::Value::as_str)
            == Some("GRAPHQL_VALIDATION_FAILED")
        && error.message().starts_with("Cannot query field ")
}

type Outcome = Option<Result<(), QueryError>>;

enum Entry {
    Loaded,
    Loading(watch::Receiver<Outcome>),
}

/// What a request must do before it may rely on its module.
pub(crate) enum Turn<'a> {
    /// The session already serves the module.
    Loaded,
    /// Another request is loading the module.
    Wait(LoadWaiter),
    /// This request loads the module.
    Drive(LoadDriver<'a>),
}

/// One session's module loads.
#[derive(Default)]
pub(crate) struct ModuleLoads {
    entries: Mutex<HashMap<ModuleRequirement, Entry>>,
}

impl ModuleLoads {
    /// Claims the load of `requirement`, or joins the one in progress.
    ///
    /// The lock is held only to read or change the entry, never while a request is in
    /// flight, so loads of different modules and requests on loaded ones do not wait
    /// for each other.
    pub(crate) fn turn(&self, requirement: &ModuleRequirement) -> Turn<'_> {
        let mut entries = self.lock();
        match entries.get(requirement) {
            Some(Entry::Loaded) => Turn::Loaded,
            Some(Entry::Loading(outcome)) => Turn::Wait(LoadWaiter {
                outcome: outcome.clone(),
            }),
            None => {
                let (sender, receiver) = watch::channel(None);
                entries.insert(requirement.clone(), Entry::Loading(receiver));
                Turn::Drive(LoadDriver {
                    loads: self,
                    requirement: requirement.clone(),
                    outcome: Some(sender),
                })
            }
        }
    }

    fn lock(&self) -> MutexGuard<'_, HashMap<ModuleRequirement, Entry>> {
        // Every critical section leaves the map consistent, so a panic elsewhere in a
        // thread that held the lock does not invalidate it.
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A request's place behind the load that another request drives.
pub(crate) struct LoadWaiter {
    outcome: watch::Receiver<Outcome>,
}

impl LoadWaiter {
    /// Waits for the load and returns its outcome, or `None` when the request driving
    /// it was dropped first. The caller then takes another turn.
    ///
    /// Dropping this future affects no other request.
    pub(crate) async fn outcome(mut self) -> Option<Result<(), QueryError>> {
        let outcome = self.outcome.wait_for(Option::is_some).await.ok()?;
        outcome.clone()
    }
}

/// The claim on one module's load, held by the request that performs it.
pub(crate) struct LoadDriver<'a> {
    loads: &'a ModuleLoads,
    requirement: ModuleRequirement,
    outcome: Option<watch::Sender<Outcome>>,
}

impl LoadDriver<'_> {
    /// Records the load's result and hands it to every waiting request.
    ///
    /// A failure clears the entry before the waiters learn of it, so the next request
    /// after any of them starts a fresh load.
    pub(crate) fn finish(mut self, result: Result<(), QueryError>) -> Result<(), QueryError> {
        {
            let mut entries = self.loads.lock();
            match &result {
                Ok(()) => entries.insert(self.requirement.clone(), Entry::Loaded),
                Err(_) => entries.remove(&self.requirement),
            };
        }
        if let Some(outcome) = self.outcome.take() {
            // No receiver means no request is waiting, which is not a failure.
            let _ = outcome.send(Some(result.clone()));
        }
        result
    }
}

impl Drop for LoadDriver<'_> {
    fn drop(&mut self) {
        if self.outcome.is_some() {
            // The driving request was dropped mid-load. Clearing the entry before the
            // sender closes lets the first waiter to wake claim the load.
            self.loads.lock().remove(&self.requirement);
        }
    }
}
