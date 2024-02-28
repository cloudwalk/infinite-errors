//! Generic error handling framework with static backtraces.

use std::{
    fmt::{self, Display, Formatter},
    panic::Location,
};

#[cfg(test)]
mod test;

pub use infinite_errors_macros::err_context;

/// Generic eror type with backtrace.
///
/// The generic argument `K` is the application-specific error kind.
#[derive(Debug, derive_more::Error)]
pub struct Error<K> {
    kind: K,
    cause: Option<Box<Error<K>>>,
    location: &'static Location<'static>,
}

impl<K> Error<K> {
    /// Create a new [Error] from an error kind and an error [Location].
    pub fn new(kind: K, location: &'static Location<'static>) -> Self {
        Self {
            kind,
            cause: None,
            location,
        }
    }

    /// Get the internal error kind.
    pub fn kind(&self) -> &K {
        &self.kind
    }

    /// Get the cause for this error, if one exists.
    pub fn cause(&self) -> Option<&Self> {
        self.cause.as_deref()
    }

    /// Get the location where this [Error] was constructed.
    pub fn location(&self) -> &'static Location<'static> {
        self.location
    }
}

impl<K> Display for Error<K>
where
    K: Display,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "[{}] {}", self.location, self.kind)?;
        if let Some(cause) = &self.cause {
            write!(f, ": {cause}")?;
        }

        Ok(())
    }
}

impl<K> From<K> for Error<K> {
    #[track_caller]
    fn from(kind: K) -> Self {
        Self::new(kind, Location::caller())
    }
}

/// Helper trait to add context to errors.
pub trait ErrorContext<T, K> {
    /// Add an error kind to the top of the error backtrace.
    #[track_caller]
    fn err_context(self, kind: K) -> Result<T, Error<K>>;

    /// Add an error kind returned by a function to the top of the error
    /// backtrace. The function should only be called if `self` is indeed an
    /// error.
    #[track_caller]
    fn err_context_with(self, kind: impl FnOnce() -> K) -> Result<T, Error<K>>;
}

impl<T, E, K> ErrorContext<T, K> for Result<T, E>
where
    E: Into<Error<K>>,
{
    fn err_context(self, kind: K) -> Result<T, Error<K>> {
        self.map_err(|x| Error {
            kind,
            cause: Some(Box::new(x.into())),
            location: Location::caller(),
        })
    }

    fn err_context_with(self, f: impl FnOnce() -> K) -> Result<T, Error<K>> {
        self.map_err(|x| Error {
            kind: f(),
            cause: Some(Box::new(x.into())),
            location: Location::caller(),
        })
    }
}
