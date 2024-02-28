use assert_matches::assert_matches;
use derive_more::{Display, From};

use crate::{Error, ErrorContext};

#[derive(Debug, Display, From)]
enum TestErrorKind {
    Context,
    BaseError(&'static str),
}

type TestResult = Result<(), Error<TestErrorKind>>;

#[test]
fn err_context_ok() {
    TestResult::Ok(())
        .err_context(TestErrorKind::Context)
        .unwrap();
}

#[test]
fn err_context_err() {
    let err = TestResult::Err(Error::<TestErrorKind>::from(TestErrorKind::BaseError(
        "test",
    )))
    .err_context(TestErrorKind::Context)
    .unwrap_err();

    assert_matches!(err.kind(), TestErrorKind::Context);
    assert_matches!(
        err.cause().unwrap().kind(),
        TestErrorKind::BaseError("test")
    );
    assert_matches!(err.cause().unwrap().cause(), None);
}

#[test]
fn err_context_with_err() {
    let err = TestResult::Err(Error::<TestErrorKind>::from(TestErrorKind::BaseError(
        "test",
    )))
    .err_context_with(|| TestErrorKind::Context)
    .unwrap_err();

    assert_matches!(err.kind(), TestErrorKind::Context);
    assert_matches!(
        err.cause().unwrap().kind(),
        TestErrorKind::BaseError("test")
    );
    assert_matches!(err.cause().unwrap().cause(), None);
}
