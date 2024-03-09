# infinte-tracing

Uses `minitrace` and Cloudwalk's `logcall` to allow logs to be generated in the Google's "stack driver"
format, outperforming `tracing-subscriber`, `trace-stackdriver` and even Cloudwalk's `trace-stackdrive`,
as demonstrated in the benchmarks.

## Usage

For those coming from `tracing-subscriber`, annotations like this are familiar:

```nocompile
#[tracing::instrument(err, skip(password))]
fn log_calls_on_err_skipping_the_password_parameter(
    &self,
    a: u32,
    password: &str,
) -> Result<bool, QueryError> {
    ...
}
```

Variations include using `skipall` and, optionally, providing a `target` value.

This crate mimics the exact same parameters, provided that you use:

```nocompile
use infinite-tracing::tracing;
```


### Creating new Spans

This crate uses the concept of `trace_id`: a `u128` number used to group together logs events. It is suggested that
the users of this library do inform when a new request is being processed.

The grouping of log events can be divided in 2 scenarios:
  1) The processing of information is starting now -- we are safe to create a new `trace_id`
  2) We are continuing the information processing started elsewhere -- we want to inform a pre-existing `trace_id` for our logs

In any case, we call this "opening a new span" -- and this is how we can distinguish between the scenarios:
```nocompile
    let _guard = infinite_tracing::new_span_from_u128_trace_id(full_name!(), trace_id);         // use this if you have a `128` `trace_id`
    let _guard = infinite_tracing::new_span_from_google_trace_id(full_name!(), trace_id);       // use this if you have a `trace_id` in a `String` form like "c951b27d3c8aa7fb6ca4aee909085ea1/1186820540535753586" -- as used by GCP.
    let _guard = infinite_tracing::new_span_from_uuid(full_name!(), trace_id);                  // use this if you have a `trace_id` in a `String` form like "b1da93b7-0c34-42e6-be2b-bb14bef0a891" -- as used by the `uuid` crate.
    let (trace_id, _guard) = infinite_tracing::new_span_with_random_trace_id(full_name!());  // use this if you are starting a new processing. The created `trace_id` may be shared with other services.
```

Please, note that the caller must retain `_guard` until the processing ends.


## Features

The following features control the behavior of this crate:

  - `disabled` -- turns this crate completely off. Useful for debugging issues or for cheap performance gains, at the expense of turning off logs;
  - `u128_trace_id` -- outputs `trace_id` as `u128`;
  - `gcp_trace_id` -- outputs `trace_id` in the GCP format -- like `c951b27d3c8aa7fb6ca4aee909085ea1/1186820540535753586`;
  - `uuid_trace_id` -- outputs `trace_id` as a string returned by the `uuid` crate;
  - `structured-logger` -- uses the `structured-logger` crate as the logging backend -- the only one supported by now.

If no trace_id options are specified, the output will be done in the GCP format.


## Mandatory Dependencies

Projects using this crate should define these dependencies in their `Cargo.toml`:

```nocompile
[dependencies]

# `infinite-tracing` mandatory dependencies
###########################################

# the fastest tracing library
minitrace = { version = "0.6", features = ["enable"] }
# allows instrumenting function calls -- inputs & outputs
logcall   = { git = "https://github.com/cloudwalk/logcall", branch = "log_inputs", features="structured-logging" }
# allows structured fields along with the `log` messages
structured-logger = "1"
log = "0.4"
```


## User Project Setup

This crate must be setup & teared down, in your project, like this:

```nocompile
fn main() {
    setup_infinite_tracing(BufWriter::with_capacity(32768, std::io::stdout()));
    // (...)
    infinite_tracing::teardown_intinite_tracing();
}
```