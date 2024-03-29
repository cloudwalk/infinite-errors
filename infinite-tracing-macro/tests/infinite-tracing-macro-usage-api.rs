mod common;
use common::*;

use infinite_tracing::instrument;
use log::info;

#[test]
fn standard_usage() {
    #[instrument]
    fn do_something(a: u32) -> Result<u32, u32> {
        info!(a=42; "Here I am!");
        Err(a)
    }

    let expected_logs = [
        "{\"logging.googleapis.com/sourceLocation\":{\"FILE\":\"\",\"LINE\":\"\"},\"message\":\"Here I am!\",\"severity\":\"INFO\",\"span\":{\"a\":\"42\"},\"target\":\"infinite_tracing_macro_usage_api::standard_usage::do_something\""
    ];

    let collect_logs = follow_logs();
    _ = do_something(10);
    let observed_logs = collect_logs()
        .into_iter()
        .map(|l| normalize_log(&l))
        .collect::<Vec<String>>();
    assert_eq!(observed_logs, expected_logs, "Wrong log contents");
}

#[test]
fn log_result_and_params_on_err() {
    #[instrument(err)]
    fn do_something(a: u32) -> Result<u32, u32> {
        Err(a)
    }

    let expected_logs = [
        "{\"logging.googleapis.com/sourceLocation\":{\"FILE\":\"infinite-tracing-macro/tests/infinite-tracing-macro-usage-api.rs\",\"LINE\":\"30\"},\"message\":\"do_something(a: 11) => Err(11)\",\"severity\":\"ERROR\",\"span\":{\"a\":\"11\",\"module\":\"infinite_tracing_macro_usage_api\",\"ret\":\"Err(11)\"},\"target\":\"test method\"",
    ];

    let collect_logs = follow_logs();
    _ = do_something(11);
    let observed_logs = collect_logs()
        .into_iter()
        .map(|l| normalize_log(&l))
        .collect::<Vec<String>>();
    assert_eq!(observed_logs, expected_logs, "Wrong log contents");
}

#[test]
fn log_result_but_no_params_on_err() {
    #[instrument(err, skip_all)]
    fn do_something(a: u32) -> Result<u32, u32> {
        Err(a)
    }

    let expected_logs = [
        "{\"logging.googleapis.com/sourceLocation\":{\"FILE\":\"infinite-tracing-macro/tests/infinite-tracing-macro-usage-api.rs\",\"LINE\":\"50\"},\"message\":\"do_something(a: <skipped>) => Err(12)\",\"severity\":\"ERROR\",\"span\":{\"module\":\"infinite_tracing_macro_usage_api\",\"ret\":\"Err(12)\"},\"target\":\"test method\"",
    ];

    let collect_logs = follow_logs();
    _ = do_something(12);
    let observed_logs = collect_logs()
        .into_iter()
        .map(|l| normalize_log(&l))
        .collect::<Vec<String>>();
    assert_eq!(observed_logs, expected_logs, "Wrong log contents");
}

#[test]
fn log_result_and_some_params_on_err() {
    #[instrument(err, skip(_password, _secret))]
    fn do_something(a: u32, _password: u32, _secret: u32) -> Result<u32, u32> {
        Err(a)
    }

    let expected_logs = [
        "{\"logging.googleapis.com/sourceLocation\":{\"FILE\":\"infinite-tracing-macro/tests/infinite-tracing-macro-usage-api.rs\",\"LINE\":\"70\"},\"message\":\"do_something(a: 13, _password: <skipped>, _secret: <skipped>) => Err(13)\",\"severity\":\"ERROR\",\"span\":{\"a\":\"13\",\"module\":\"infinite_tracing_macro_usage_api\",\"ret\":\"Err(13)\"},\"target\":\"test method\"",
    ];

    let collect_logs = follow_logs();
    _ = do_something(13, 1, 2);
    let observed_logs = collect_logs()
        .into_iter()
        .map(|l| normalize_log(&l))
        .collect::<Vec<String>>();
    assert_eq!(observed_logs, expected_logs, "Wrong log contents");
}

/// Takes out the varying `time` and `traceId` fields of the log line
fn normalize_log(log_line: &str) -> String {
    log_line
        .split(r#","time":"#)
        .next()
        .expect("Invalid log line")
        .to_string()
}
