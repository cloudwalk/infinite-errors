use std::fmt::{Display, Formatter};
use std::task::Context;
use infinite_tracing::*;
use log::info;
use minitrace::collector::{SpanContext, SpanId, TraceId};
use tracing::instrument::WithSubscriber;

#[instrument(ret)]
fn log_all_calls_and_returns(a: u64, b: u64) -> Result<u64, Box<dyn std::error::Error>> {
    info!(intermediate=42; "A silly structured log");
    Ok(a + b)
}

#[instrument(ret, skip(_c))]
fn log_all_calls_unwanted_parameters<Any>(
    a: u64,
    b: u64,
    _c: Any,
) -> Result<u64, Box<dyn std::error::Error>> {
    Ok(a + b)
}

#[instrument(err)]
fn log_error_calls_only(a: u64, b: u64) -> Result<u64, Box<dyn std::error::Error>> {
    a.checked_sub(b)
        .ok_or_else(|| Box::from(format!("Couldn't subtract {b} from {a}")))
}

// Uncomment the following function to see a demonstration of this library's debug capabilities
// #[instrument(err, debug)]
// fn fail_to_compile_but_outputs_the_real_code(a: u64, b: u64) -> Result<u64, Box<dyn std::error::Error>> {
//     a.checked_sub(b)
//         .ok_or_else(|| Box::from(format!("Couldn't subtract {b} from {a}")))
// }

#[instrument]
fn sync_database_function() {
    info!("This is the SYNC database function: `log_error_calls()` will be called, but it will produce an Ok Result -- so no logs should follow. Nonetheless, this log should include the `ctx` variable set on the upstream function");
    _ = log_error_calls_only(10, 3);
}

#[instrument]
fn sync_business_logic_function() {
    // add something to the span -- to be included in the logs made downstream:
    #[derive(Debug)]
    struct Context {
        state: String,
        metrics: u32,
    }
    let ctx = Context {
        state: "My Context has a State".to_string(),
        metrics: 42,
    };
    info!(ctx:?; "Context is shared here. See 'span'");
    sync_database_function();
}

#[instrument]
fn sync_endpoint_function() {
    let _guard = new_span_with_random_trace_id(full_name!());

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
    let gcp_trace_id = "c951b27d3c8aa7fb6ca4aee909085ea1/1186820540535753586";
    let _guard = new_span_from_gcp_trace_id(full_name!(), gcp_trace_id);
    async_business_logic_function().await;
    let a = 1; //Result::<u32, u32>::Ok(1);
    let b = 2;
    info!(a, b; "This is the ASYNC endpoint function -- original trace_id_string is '{gcp_trace_id}'");
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
