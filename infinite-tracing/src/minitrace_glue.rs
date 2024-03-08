use std::marker::PhantomData;
use minitrace::collector::{Config, Reporter, SpanRecord};
use uuid::Uuid;
use serde_json::json;


pub fn setup_minitrace(output_fn: impl std::io::Write + Send + 'static) {
    let json_reporter = JsonReporter::new(output_fn);
    minitrace::set_reporter(json_reporter, Config::default());
}

pub fn teardown_minitrace() {
    minitrace::flush();
}


pub struct JsonReporter<WriteImpl: std::io::Write> {
    writer: WriteImpl,
}

impl<WriteImpl: std::io::Write> JsonReporter<WriteImpl> {

    pub fn new(writer: WriteImpl) -> Self {
        Self {
            writer,
        }
    }
}

impl<WriteImpl: std::io::Write + Send + 'static> Reporter for JsonReporter<WriteImpl> {
    fn report(&mut self, spans: &[SpanRecord]) {
        for span in spans {
            let trace_id = span.trace_id.0;
            let trace_id = Uuid::from_u128(trace_id).to_string();
            for event in &span.events {
                let target = &span.name;
                let severity = &event.name;
                let mut message = "<MISSING MESSAGE>";
                let mut timestamp = "<MISSING TIMESTAMP>";
                let mut file = "";
                let mut line = "";
                let mut structured_fields = serde_json::Map::new();
                for (property_key, property_value) in &event.properties {
                    match property_key.as_ref() {
                        "timestamp" => timestamp = property_value,
                        "message" => message = property_value,
                        "file" => file = property_value,
                        "line" => line = property_value,
                        _ => { structured_fields.insert(property_key.to_string(), json!(property_value)); },
                    }
                }
                let log_line = json!({
                    "time": timestamp,
                    "target": span.name,
                    "logging.googleapis.com/sourceLocation": {"FILE": file, "LINE": line},
                    "span": structured_fields,
                    "traceId": trace_id,
                    "severity": event.name,
                    "message": message,
                });
                let mut write_op = || {
                    serde_json::to_writer(&mut self.writer, &log_line)?;
                    self.writer.write(&[b'\n'])
                };
                write_op().expect("`infinite-tracing`: `minitrace` glue: Writer errored out");
            }
        }
    }
}