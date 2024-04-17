use log::kv::{Key, Value};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::io::{Error, Write};
use std::iter::IntoIterator;
use once_cell::sync::Lazy;


static LEVEL_KEY: Lazy<Key> = Lazy::new(|| Key::from_str("level"));
static TARGET_KEY: Lazy<Key> = Lazy::new(|| Key::from_str("target"));
static CHRONO_FORMAT: Lazy<Vec<chrono::format::Item>> = Lazy::new(|| chrono::format::StrftimeItems::new("%Y-%m-%dT%H:%M:%S%.3fZ").into_iter().collect());

pub fn setup_structured_logger() {
    struct MyWriter;
    impl structured_logger::Writer for MyWriter {
        fn write_log(&self, entries: &BTreeMap<Key, Value>) -> Result<(), Error> {
            let level = entries
                .get(&*LEVEL_KEY)
                .map(|value| value.to_string())
                .unwrap_or_else(|| String::from("<MISSING LEVEL>"));
            // Add the log record as an event in the current local span
            minitrace::Event::add_to_local_parent(level, || {
                #[allow(clippy::needless_borrowed_reference)]
                entries
                    .iter()
                    .filter(|(&ref k, _v)| {
                        k != &*LEVEL_KEY && k != &*TARGET_KEY
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
                            let timestamp =
                                chrono_time.format_with_items(CHRONO_FORMAT.iter()).to_string();
                            (Cow::Borrowed("timestamp"), timestamp)
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
