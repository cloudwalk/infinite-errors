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

## Setup

The logger must be setup, in your project, like this:

```nocompile
    // setup structured-logger
    // setup minitrace
```