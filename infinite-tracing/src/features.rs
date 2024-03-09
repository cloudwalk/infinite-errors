//! Abstraction over the Cargo.toml features

/// Converts our `u128` trace_id into a `Uuid`
#[cfg(feature = "uuid_trace_id")]
pub fn convert_trace_id(trace_id: u128) -> String {
    uuid::Uuid::from_u128(trace_id).to_string()
}

/// simpy uses our `u128` trace_id, without conversions
#[cfg(feature = "u128_trace_id")]
pub fn convert_trace_id(trace_id: u128) -> String {
    trace_id.to_string()
}

/// Converts our `u128` trace_id into a `Uuid`
#[cfg(any(feature = "gcp_trace_id", not(any(feature = "uuid_trace_id", feature = "u128_trace_id"))))]
pub fn convert_trace_id(trace_id: u128) -> String {
    crate::u128_to_gcp_trace_id(trace_id)
}