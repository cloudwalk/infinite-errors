//! Generic error handling framework with static backtraces.

use std::panic::Location;

pub use derive_more::Error;
pub use infinite_errors_macros::err_context;

/// Generate a rich error type using a given error kind.
///
/// The type will be called `Error`. Also generates an `ErrorContext` trait
/// similar to [ErrorContext] but specialized for this new error type.
///
/// The reason why we cannot define an error type in this crate and export it
/// is because orphan rules would make the `?` operator more awkward to use.
///
/// # Usage
///
/// Define your error kind and then call this macro with that error kind as
/// argument:
///
/// ```ignore
/// declare_error_type!(ErrorKind);
/// ```
///
/// Both the error type and the generated `ErrorContext` trait will be `pub`.
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
                write!(f, "{}", self.kind)?;
                if let Some(cause) = &self.cause {
                    write!(f, ": {cause}")?;
                }

                Ok(())
            }
        }

        impl<T> ::std::convert::From<T> for Error
        where
            T: ::std::convert::Into<$error_kind>,
        {
            #[track_caller]
            fn from(kind: T) -> Self {
                Self::new(kind.into(), ::std::panic::Location::caller())
            }
        }

        /// Helper trait to add context to errors.
        pub trait ErrorContext<T> {
            /// Add an error kind to the top of the error backtrace.
            #[track_caller]
            fn err_context(self, kind: $error_kind) -> ::std::result::Result<T, Error>;

            /// Add an error kind returned by a function to the top of the error
            /// backtrace. The function should only be called if `self` is indeed an
            /// error.
            #[track_caller]
            fn err_context_with(
                self,
                kind: impl FnOnce() -> $error_kind,
            ) -> ::std::result::Result<T, Error>;
        }

        impl<T, OE> ErrorContext<T> for ::std::result::Result<T, OE>
        where
            OE: Into<Error>,
        {
            fn err_context(self, kind: $error_kind) -> ::std::result::Result<T, Error> {
                self.map_err(|x| Error {
                    kind,
                    cause: ::std::option::Option::Some(::std::boxed::Box::new(x.into())),
                    location: ::std::panic::Location::caller(),
                })
            }

            fn err_context_with(
                self,
                f: impl FnOnce() -> $error_kind,
            ) -> ::std::result::Result<T, Error> {
                self.map_err(|x| Error {
                    kind: f(),
                    cause: ::std::option::Option::Some(::std::boxed::Box::new(x.into())),
                    location: ::std::panic::Location::caller(),
                })
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
///
/// Most likely you want to use the trait of the same name and API generated
/// by [declare_error_type].
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

    fn err_context_with(self, f: impl FnOnce() -> K) -> Result<T, E> {
        self.map_err(|x| E::new(f(), Some(Box::new(x.into())), Location::caller()))
    }
}
