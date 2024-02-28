use assert_matches::assert_matches;
use derive_more::{Display, From};

use infinite_errors::{err_context, Error};

const BASE_ERROR_MESSAGE: &str = "test";

#[derive(Debug, Display, From)]
enum TestErrorKind {
    Context,
    BaseError(&'static str),
}

type TestResult = Result<(), Error<TestErrorKind>>;

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
