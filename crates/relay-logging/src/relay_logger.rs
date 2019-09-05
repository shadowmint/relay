use crate::relay_log_entry::RelayLogEntry;
use crate::relay_log_format::RelayLogFormat;
use base_logging::loggers::ConsoleLogger;
use base_logging::{Level, Loggable, LoggerRef};

pub(crate) const CONTEXT: &'static str = "context";
pub(crate) const CONTEXT_DEFAULT: &'static str = "Unknown";

#[derive(Clone)]
pub struct RelayLogger {
    context: String,
    logger: LoggerRef,
}

impl RelayLogger {
    pub fn new(context: &str) -> RelayLogger {
        RelayLogger {
            context: context.to_string(),
            logger: LoggerRef::new()
                // TODO: Make this async in the implementation OH YEAH
                .with(ConsoleLogger::new())
                .with_format(RelayLogFormat::new())
                .with_level(Level::Info),
        }
    }

    pub fn trace(&self, message: impl Loggable) {
        self.logger
            .log(Level::Trace, RelayLogEntry::new(&self.context, &message));
    }

    pub fn debug(&self, message: impl Loggable) {
        self.logger
            .log(Level::Debug, RelayLogEntry::new(&self.context, &message));
    }

    pub fn info(&self, message: impl Loggable) {
        self.logger
            .log(Level::Info, RelayLogEntry::new(&self.context, &message));
    }

    pub fn warn(&self, message: impl Loggable) {
        self.logger
            .log(Level::Warn, RelayLogEntry::new(&self.context, &message));
    }

    pub fn error(&self, message: impl Loggable) {
        self.logger
            .log(Level::Error, RelayLogEntry::new(&self.context, &message));
    }
}
