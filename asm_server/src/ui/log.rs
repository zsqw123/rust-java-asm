use log::{Level, Metadata, Record};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

pub struct LogHolder {
    pub min_level: Level,
    pub max_len: usize,
    pub records: Mutex<VecDeque<SimpleRecord>>,
}

impl Default for LogHolder {
    fn default() -> Self {
        Self {
            min_level: Level::Info,
            max_len: 1000,
            records: Mutex::new(VecDeque::with_capacity(1000)),
        }
    }
}

pub struct SimpleRecord {
    pub level: Level,
    pub message: Arc<str>,
}

impl log::Log for LogHolder {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.min_level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let deque = &mut self.records.lock().unwrap();
        if deque.len() >= self.max_len {
            deque.pop_front();
        }
        deque.push_back(SimpleRecord {
            level: record.level(),
            message: Arc::from(record.args().to_string()),
        });
    }

    fn flush(&self) {}
}

pub fn inject_log(log_holder: Arc<LogHolder>) {
    log::set_boxed_logger(Box::new(log_holder)).unwrap();
    log::set_max_level(Level::Trace.to_level_filter());
}
