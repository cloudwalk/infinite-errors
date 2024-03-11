use minitrace::local::LocalParentGuard;
use minitrace::prelude::SpanContext;
use minitrace::Span;
use once_cell::sync::Lazy;
use parking_lot::lock_api::RawMutex as _RawMutexInit;
use parking_lot::{Mutex, RawMutex};
use std::sync::Arc;
use std::time::Duration;

/// Initializes `minitrace` and exposes the `log_collector_generator` function, which is used by [follow_logs()].
static LOG_COLLECTOR_GENERATOR: Lazy<Box<GeneratorFn>> = Lazy::new(|| {
    let reporter = LogCollector::new();
    let generate_log_collector = reporter.log_collector_generator();
    infinite_tracing::setup_infinite_tracing(reporter.writer());
    Box::new(generate_log_collector)
});
type GeneratorFn = dyn Fn(Span, LocalParentGuard) -> Box<dyn FnOnce() -> Vec<String>> + Send + Sync;

/// Tests should call this to start capturing log events.\
/// The returned value is a closure that should be called to
/// collect the emitted log events, for assertion.
pub fn follow_logs() -> impl FnOnce() -> Vec<String> {
    let generate_log_collector = LOG_COLLECTOR_GENERATOR.as_ref(); // tricky line: causes the static initialization code to run **before** we call the next `minitrace` functions
    let root_span = Span::root("test method", SpanContext::random());
    let guard = root_span.set_local_parent();
    generate_log_collector(root_span, guard)
}

/// Tricky struct to allow capturing `minitrace` log events, for test assertions:
///   1) As this struct is instantiated, [Self::log_collector_generator()] must be called;
///   2) This struct's instance can be passed to `minitrace`, as it requires so;
///   3) Tests should start by calling the output of #1. #1 is a function and, by calling it, you will
///      get a "log collector", which is yet another function;
///   4) Before the assertions, tests should call the "log collector", finally obtaining the
///      vector of Strings with the log lines generated.
///
/// This rather complex workflow is required by the way `minitrace` is implemented,
/// requiring some guards and spans to be kept and dropped at certain times, so the
/// logs can be appropriately flushed.
pub struct LogCollector {
    /// Logs will be put here
    collected: Arc<Mutex<Vec<String>>>,
    /// This remains locked from when a log collector is generated
    /// up until the logs are actually collected, meaning tests
    /// cannot run concurrently as we must have strict control
    /// of what is being logged -- which must be done serially...
    usage_lock: Arc<RawMutex>,
}

impl Default for LogCollector {
    fn default() -> Self {
        Self {
            collected: Arc::new(Mutex::new(Vec::new())),
            usage_lock: Arc::new(RawMutex::INIT),
        }
    }
}

impl LogCollector {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gives out a `Write` implementation that will put lines into [Self::collected].
    pub fn writer(&self) -> impl std::io::Write {
        struct InternalWriter {
            buffer: Vec<u8>,
            collected: Arc<Mutex<Vec<String>>>,
        }
        impl std::io::Write for InternalWriter {
            fn write(&mut self, buff: &[u8]) -> Result<usize, std::io::Error> {
                self.buffer.extend_from_slice(buff);
                while let Some(pos) = self.buffer.iter().position(|&c| c == b'\n') {
                    let line = String::from_utf8_lossy(&self.buffer[0..=pos]).to_string();
                    self.buffer.drain(0..=pos);
                    self.collected.lock().push(line);
                }
                Ok(buff.len())
            }
            fn flush(&mut self) -> Result<(), std::io::Error> {
                if !self.buffer.is_empty() {
                    let last_line = String::from_utf8_lossy(&self.buffer).to_string();
                    self.collected.lock().push(last_line);
                }
                Ok(())
            }
        }

        InternalWriter {
            buffer: Vec::with_capacity(128),
            collected: self.collected.clone(),
        }
    }

    /// Weird API needed for `minitrace` to do its shenanigans: [Self::log_collector()] can only do its
    /// business after dropping minitrace's `span` and `guard`
    pub fn log_collector_generator(
        &self,
    ) -> impl Fn(
        /*span: */ Span,
        /*guard: */ LocalParentGuard,
    ) -> Box<dyn FnOnce() -> Vec<String>> {
        let usage_lock = self.usage_lock.clone();
        let collected = self.collected.clone();
        move |span, guard| Box::new(Self::log_collector(&usage_lock, &collected, span, guard))
    }

    fn log_collector(
        usage_lock: &Arc<RawMutex>,
        collected: &Arc<Mutex<Vec<String>>>,
        span: Span,
        guard: LocalParentGuard,
    ) -> impl FnOnce() -> Vec<String> {
        let usage_lock = usage_lock.clone();
        // the log collector was called: a new test started:
        usage_lock.lock();
        collected.lock().clear();
        let collected = Arc::clone(collected);
        move || {
            drop(span);
            drop(guard);
            minitrace::flush();
            std::thread::sleep(Duration::from_millis(1)); // really needed?
            let collected = collected.lock().clone();
            unsafe { usage_lock.unlock() };
            collected
        }
    }
}
