use assert_matches::assert_matches;
use derive_more::{Display, From};

use infinite_errors::{declare_error_type, err_context};

const BASE_ERROR_MESSAGE: &str = "test";

#[derive(Debug, Display, From)]
pub enum TestErrorKind {
    Context,
    BaseError(&'static str),
}

declare_error_type!(TestErrorKind);

type TestResult = Result<(), Error>;

#[test]
fn err_context_ok() {
    let res: TestResult = TestResult::Ok(()).err_context(TestErrorKind::Context);
    res.unwrap();
}

#[test]
fn err_context_err() {
    let err: Error = TestResult::Err(Error::from(TestErrorKind::BaseError("test")))
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
    let err: Error = TestResult::Err(Error::from(TestErrorKind::BaseError("test")))
        .err_context_with(|| TestErrorKind::Context)
        .unwrap_err();

    assert_matches!(err.kind(), TestErrorKind::Context);
    assert_matches!(
        err.cause().unwrap().kind(),
        TestErrorKind::BaseError("test")
    );
    assert_matches!(err.cause().unwrap().cause(), None);
}

#[test]
fn err_context_macro_fn() {
    #[err_context(TestErrorKind::Context)]
    fn test() -> TestResult {
        Err(TestErrorKind::BaseError(BASE_ERROR_MESSAGE))
    }

    assert_correct_error_context(test());
}

#[test]
fn err_context_macro_fn_try_return() {
    #[err_context(TestErrorKind::Context)]
    fn test() -> TestResult {
        Err(TestErrorKind::BaseError(BASE_ERROR_MESSAGE))?;

        TestResult::Ok(())
    }

    assert_correct_error_context(test());
}

#[test]
fn err_context_macro_async_fn() {
    #[err_context(TestErrorKind::Context)]
    async fn test() -> TestResult {
        Err(TestErrorKind::BaseError(BASE_ERROR_MESSAGE))
    }

    assert_correct_error_context(futures_executor::block_on(test()));
}

#[test]
fn err_context_macro_async_fn_try_return() {
    #[err_context(TestErrorKind::Context)]
    async fn test() -> TestResult {
        Err(TestErrorKind::BaseError(BASE_ERROR_MESSAGE))?;

        TestResult::Ok(())
    }

    assert_correct_error_context(futures_executor::block_on(test()));
}

fn assert_correct_error_context(res: TestResult) {
    let err = res.unwrap_err();

    assert_matches!(err.kind(), TestErrorKind::Context);
    assert_matches!(
        err.cause().unwrap().kind(),
        TestErrorKind::BaseError(BASE_ERROR_MESSAGE)
    );
    assert_matches!(err.cause().unwrap().cause(), None);
}
