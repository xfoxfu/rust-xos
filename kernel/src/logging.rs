use log::{LevelFilter, Metadata, Record};

pub fn initialize() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") {
        Some("error") => LevelFilter::Error,
        Some("warn") => LevelFilter::Warn,
        Some("info") => LevelFilter::Info,
        Some("debug") => LevelFilter::Debug,
        Some("trace") => LevelFilter::Trace,
        _ => LevelFilter::Trace,
    });
}

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= metadata.level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("[{:5}] {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}
