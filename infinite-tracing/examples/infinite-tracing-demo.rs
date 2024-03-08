use ctor::ctor;
use infinite_tracing::instrument;
use log::kv::{Key, Value};
use log::{debug, error, info, warn, LevelFilter};
use logcall::*;
use minitrace::collector::{Config, ConsoleReporter, Reporter};
use minitrace::prelude::*;
use minitrace::Event;
use serde::Serialize;
use serde_json::json;
use std::any::Any;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::Error;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

#[instrument(ret)]
fn log_all_calls_and_returns(a: u64, b: u64) -> Result<u64, Box<dyn std::error::Error>> {
    info!(intermediate=42; "A silly structured log"); // ==> span
    Ok(a + b) // ==> estruturado ou no message
}

#[instrument(ret, skip(c))]
fn log_all_calls_unwanted_parameters<Any>(
    a: u64,
    b: u64,
    c: Any,
) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(a + b)
}

#[instrument(err)]
fn log_error_calls_only(a: u64, b: u64) -> Result<u64, Box<dyn std::error::Error>> {
    a.checked_sub(b)
        .ok_or_else(|| Box::from(format!("Couldn't subtract {b} from {a}")))
}

#[instrument]
fn sync_database_function() {
    info!("This is the SYNC database function: `log_error_calls()` will be called, but it will produce an Ok Result -- so no logs should follow");
    _ = log_error_calls_only(10, 3);
}

#[instrument]
fn sync_business_logic_function() {
    sync_database_function();
}

#[instrument]
fn sync_endpoint_function() {
    let trace_id = Uuid::new_v4().to_string();

    let root_span = Span::root(full_name!(), SpanContext::random());
    let _guard = root_span.set_local_parent();

    // Your custom logic here
    sync_business_logic_function();

    // Log with trace_id
    info!("This is the SYNC endpoint function");
}

#[instrument]
async fn async_database_function() {
    info!("This is the ASYNC database function -- One Ok and another Err log should follow. After: another Ok with ignored parameters");
    _ = log_all_calls_and_returns(10, 11);
    _ = log_error_calls_only(10, 11);
    _ = log_all_calls_unwanted_parameters(1, 1, "DO NOT LOG THIS");
}

#[instrument]
async fn async_business_logic_function() {
    info!("This is the ASYNC business logic function");
    async_database_function().await;
}

#[instrument]
async fn async_endpoint_function() {
    let uuid = Uuid::new_v4();
    let trace_id = uuid.as_u128();
    let trace_id_string = uuid.to_string();
    let reconstructed_trace_id = Uuid::from_str(&trace_id_string)
        .unwrap_or_else(|_| Uuid::new_v4())
        .as_u128();
    debug_assert_eq!(reconstructed_trace_id, trace_id);

    // let root_span = Span::root(full_name!(), SpanContext::random());
    let root_span = Span::root(
        full_name!(),
        SpanContext::new(TraceId(trace_id), SpanId::default()),
    );
    let _guard = root_span.set_local_parent();

    async_business_logic_function().await;

    let a = 1; //Result::<u32, u32>::Ok(1);
    let b = 2;
    info!(a, b; "This is the ASYNC endpoint function -- original trace_id_string is '{trace_id_string}'");
}

#[tokio::main]
async fn main() {
    // setup tracing to log to stdout
    infinite_tracing::setup_infinite_tracing(std::io::stdout());

    sync_endpoint_function();
    async_endpoint_function().await;

    // for a graceful shutdown, ensuring no
    // log lines will be lost, always call:
    infinite_tracing::teardown_intinite_tracing();
}
