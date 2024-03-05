//! Generic error handling framework with static backtraces.

#[cfg(test)]
mod test;

use std::panic::Location;

pub use derive_more::Error;
pub use infinite_errors_macros::err_context;

#[macro_export]
macro_rules! declare_error_type {
    ($error_kind:ident) => {
        /// Generic eror type with backtrace.
        #[derive(::std::fmt::Debug, ::infinite_errors::Error)]
        pub struct Error {
            kind: $error_kind,
            cause: ::std::option::Option<::std::boxed::Box<Error>>,
            location: &'static ::std::panic::Location<'static>,
        }

        impl Error {
            /// Create a new [Error] from an error kind and an error [Location].
            pub fn new(
                kind: $error_kind,
                location: &'static ::std::panic::Location<'static>,
            ) -> Self {
                Self {
                    kind,
                    cause: ::std::option::Option::None,
                    location,
                }
            }

            /// Get the internal error kind.
            pub fn kind(&self) -> &$error_kind {
                &self.kind
            }

            /// Get the cause for this error, if one exists.
            pub fn cause(&self) -> ::std::option::Option<&Self> {
                self.cause.as_deref()
            }

            /// Get the location where this [Error] was constructed.
            pub fn location(&self) -> &'static ::std::panic::Location<'static> {
                self.location
            }
        }

        impl ::infinite_errors::ErrorType for Error {
            type ErrorKind = $error_kind;

            fn new(
                kind: Self::ErrorKind,
                cause: ::std::option::Option<::std::boxed::Box<Self>>,
                location: &'static ::std::panic::Location<'static>,
            ) -> Self {
                Self {
                    kind,
                    cause,
                    location,
                }
            }
        }

        impl ::std::fmt::Display for Error {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, "[{}] {}", self.location, self.kind)?;
                if let Some(cause) = &self.cause {
                    write!(f, ": {cause}")?;
                }

                Ok(())
            }
        }

        impl<T> From<T> for Error
        where
            T: Into<$error_kind>,
        {
            #[track_caller]
            fn from(kind: T) -> Self {
                Self::new(kind.into(), ::std::panic::Location::caller())
            }
        }
    };
}

/// Trait for error types created by [declare_error_type].
pub trait ErrorType {
    /// The `ErrorKind` type.
    type ErrorKind;

    /// Create a new [ErrorType] with the given inner kind, cause and error
    /// location.
    fn new(
        kind: Self::ErrorKind,
        cause: Option<Box<Self>>,
        location: &'static Location<'static>,
    ) -> Self;
}

/// Helper trait to add context to errors.
pub trait ErrorContext<T, K, E> {
    /// Add an error kind to the top of the error backtrace.
    #[track_caller]
    fn err_context(self, kind: K) -> Result<T, E>;

    /// Add an error kind returned by a function to the top of the error
    /// backtrace. The function should only be called if `self` is indeed an
    /// error.
    #[track_caller]
    fn err_context_with(self, kind: impl FnOnce() -> K) -> Result<T, E>;
}

impl<T, K, E, OE> ErrorContext<T, K, E> for Result<T, OE>
where
    OE: Into<E>,
    E: ErrorType<ErrorKind = K>,
{
    fn err_context(self, kind: K) -> Result<T, E> {
        self.map_err(|x| E::new(kind, Some(Box::new(x.into())), Location::caller()))
    }

    fn err_context_with(self, f: impl FnOnce() -> K) -> ::std::result::Result<T, E> {
        self.map_err(|x| E::new(f(), Some(Box::new(x.into())), Location::caller()))
    }
}
