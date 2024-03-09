mod features;
mod minitrace_glue;
mod structured_logger_glue;

use std::borrow::Cow;
use minitrace::collector::{SpanContext, SpanId, TraceId};

pub use infinite_tracing_macro::instrument;
pub use minitrace::full_name;


/// Should be executed at the application start -- once.
/// Example:
/// ```nocompile
///   use infinite_tracing::*;
///   // buffered: higher performance, traded-off for a higher latency
///   setup_infinite_tracing(BufWriter::with_capacity(32768, std::io::stdout()));
///   // unbuffered: lower latency, but many context switches
///   setup_infinite_tracing(std::io::stdout);
pub fn setup_infinite_tracing(output_fn: impl std::io::Write + Send + 'static) {
    structured_logger_glue::setup_structured_logger();
    minitrace_glue::setup_minitrace(output_fn);
}

/// Should be executed at the application shutdown -- or else some log events may be lost,
/// specially if the `Writer` given to [setup_infinite_tracing] is buffered.
pub fn teardown_intinite_tracing() {
    minitrace_glue::teardown_minitrace();
    structured_logger_glue::teardown_structured_logger();
}


/// Use this if you have a `u128` `trace_id` from a processing started elsewhere.\
/// The caller must retain the returned value until the processing ends.
pub fn new_span_from_u128_trace_id(name: impl Into<Cow<'static, str>>, trace_id: u128) -> (minitrace::prelude::Span, minitrace::local::LocalParentGuard) {
    let root_span = minitrace::Span::root(
        name,
        SpanContext::new(TraceId(trace_id), SpanId::default()),
    );
    let guard = root_span.set_local_parent();
    (root_span, guard)
}

/// Use this if you have a String `trace_id` from GCP, in the form "c951b27d3c8aa7fb6ca4aee909085ea1/1186820540535753586".\
/// Please note that this string contains 192 bits of information and that we will down-cast it to `u128`, keeping
/// the first 16 hex chars before '/' as well as the number after '/' -- the last 16 hex chars before '/' will be filled with 0
/// if you want to serialize a `u128` to a GCP trace_id. Please see [u128_to_gcp_trace_id()].
/// The caller must retain the returned value until the processing ends.
pub fn new_span_from_gcp_trace_id(name: impl Into<Cow<'static, str>>, trace_id: &str) -> (minitrace::prelude::Span, minitrace::local::LocalParentGuard) {
    let trace_id = gcp_trace_id_to_u128(trace_id).unwrap_or(0);
    new_span_from_u128_trace_id(name, trace_id)
}

/// Use this if you have a UUID String as `trace_id`, in the form "1042e8d7-fdb2-42cd-b140-9eaaa671d6c6".\
/// The caller must retain the returned value until the processing ends.
pub fn new_span_from_uuid(name: impl Into<Cow<'static, str>>, trace_id: &str) -> (minitrace::prelude::Span, minitrace::local::LocalParentGuard) {
    let trace_id = uuid_to_u128(trace_id).unwrap_or(0);
    new_span_from_u128_trace_id(name, trace_id)
}

pub fn new_span_with_random_trace_id(name: impl Into<Cow<'static, str>>) -> (minitrace::prelude::Span, minitrace::local::LocalParentGuard) {
    let trace_id = SpanContext::random().trace_id.0;
    new_span_from_u128_trace_id(name, trace_id)
}


/// Down-casts a GCP `trace_id` String, in the form "c951b27d3c8aa7fb6ca4aee909085ea1/1186820540535753586"
/// into a `u128` -- a lossy operation that can be partially reverted by [u128_to_gcp_trace_id].\
/// Please note that the GCP string contains 192 bits of information and that the lower 16 hex characters before the '/'
/// will be dropped off (or filled with 0).
pub fn gcp_trace_id_to_u128(trace_id: &str) -> Option<u128> {
    // Split the trace ID into hexadecimal and decimal parts
    let parts: Vec<&str> = trace_id.split('/').collect();
    if parts.len() != 2 {
        return None; // Invalid format
    }

    // Parse the hexadecimal part
    let hex_part = parts[0];
    let hex_value = u128::from_str_radix(hex_part, 16).ok()?;

    // Parse the decimal part
    let decimal_part = parts[1].parse::<u64>().ok()?;
    let combined_value = (hex_value >> 64 << 64) | (decimal_part as u128);

    Some(combined_value)
}

pub fn u128_to_gcp_trace_id(value: u128) -> String {
    let hex_part = format!("{:032x}", value >> 64 << 64);
    let decimal_part = (value & 0xFFFFFFFFFFFFFFFF) as u64;
    format!("{}/{}", hex_part, decimal_part)
}

/// Extracts the information of a string in the form "1042e8d7-fdb2-42cd-b140-9eaaa671d6c6"
/// into its 128 bits.
fn uuid_to_u128(uuid: &str) -> Option<u128> {
    // Remove hyphens and parse each segment
    let hex_segments: Vec<&str> = uuid.split('-').collect();
    if hex_segments.len() != 5 {
        return None; // Invalid format
    }

    // Combine segments into a u128 value
    let combined_value = format!(
        "{}{}{}{}{}",
        hex_segments[0], hex_segments[1], hex_segments[2], hex_segments[3], hex_segments[4]
    );
    let parsed_value = u128::from_str_radix(&combined_value, 16).ok()?;

    Some(parsed_value)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gcp_trace_id() {
        let original_gcp_trace_id             = "c951b27d3c8aa7fb6ca4aee909085ea1/1186820540535753586";
        let expected_reconverted_gcp_trace_id = "c951b27d3c8aa7fb0000000000000000/1186820540535753586";
        let u128_trace_id = gcp_trace_id_to_u128(original_gcp_trace_id).expect("Parsing failed");
        let observed_reconverted_gcp_trace_id = u128_to_gcp_trace_id(u128_trace_id);
        assert_eq!(observed_reconverted_gcp_trace_id, expected_reconverted_gcp_trace_id, "GCP trace id conversion functions are wrong");
    }

    #[test]
    fn uuid() {
        let uuid = "550e8400-e29b-41d4-a716-446655440000";
        let expected_u128 = 113059749145936325402354257176981405696_u128;
        let observed_u128 = uuid_to_u128(uuid).expect("Parsing failed");
        assert_eq!(observed_u128, expected_u128, "UUID conversion functions are wrong");
    }
}