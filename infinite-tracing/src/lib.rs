mod structured_logger_glue;
mod minitrace_glue;

pub use infinite_tracing_macro::instrument;

pub fn setup_infinite_tracing(output_fn: impl std::io::Write + Send + 'static) {
    structured_logger_glue::setup_structured_logger();
    minitrace_glue::setup_minitrace(output_fn);
}

pub fn teardown_intinite_tracing() {
    minitrace_glue::teardown_minitrace();
    structured_logger_glue::teardown_structured_logger();
}