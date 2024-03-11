use log::kv::{Key, Value};
use minitrace::Event;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::{Error, Write};

pub fn setup_structured_logger() {
    struct MyWriter;
    impl structured_logger::Writer for MyWriter {
        fn write_log(&self, entries: &BTreeMap<Key, Value>) -> Result<(), Error> {
            let level = entries
                .get(&Key::from_str("level"))
                .map(|value| value.to_string())
                .unwrap_or(String::from("<MISSING LEVEL>"));
            // Add the log record as an event in the current local span
            Event::add_to_local_parent(level, || {
                #[allow(clippy::needless_borrowed_reference)]
                entries
                    .iter()
                    .filter(|(&ref k, _v)| {
                        k != &Key::from_str("level") && k != &Key::from_str("target")
                    })
                    .map(|(k, v)| match k.as_str() {
                        "message" => (Cow::Borrowed("message"), v.to_string()),
                        "timestamp" => {
                            let timestamp_ms = v.to_u64().unwrap_or(u64::MAX);
                            #[allow(deprecated)]
                            let chrono_time = chrono::NaiveDateTime::from_timestamp_opt(
                                (timestamp_ms / 1_000) as i64,
                                1_000_000 * (timestamp_ms % 1_000) as u32,
                            )
                            .unwrap_or(chrono::NaiveDateTime::MIN);
                            let timestamp_str =
                                chrono_time.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string();
                            (Cow::Borrowed("timestamp"), timestamp_str)
                        }
                        _ => (Cow::Owned(k.to_string()), v.to_string()),
                    })
                    .map(|(k, v)| (k, Cow::Owned(v)))
            });
            Ok(())
        }
    }
    structured_logger::Builder::new()
        .with_default_writer(Box::new(MyWriter))
        .init();
}

pub fn teardown_structured_logger() {
    std::io::stdout().flush().unwrap();
}
