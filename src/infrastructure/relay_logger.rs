use std::fmt::Debug;
use base_logging::loggers::ConsoleLogger;
use base_logging::formatters::DefaultFormatter;
use base_logging::Level;
use rust_isolate::IsolateIdentity;
use base_logging::LoggerRef;
use std::fmt::Display;

#[derive(Clone)]
pub struct RelayLogger {
    context: String,
    identity: IsolateIdentity,
    logger: LoggerRef,
}

impl RelayLogger {
    pub fn new(identity: Option<IsolateIdentity>, context: &str) -> RelayLogger {
        return RelayLogger {
            context: context.to_string(),
            identity: identity.unwrap_or(IsolateIdentity::new()),
            logger: LoggerRef::new()
                .with(ConsoleLogger::new())
                .with_format(DefaultFormatter::new())
                .with_level(Level::Info),
        };
    }

    pub fn info<T: Display>(&self, message: T) {
        self.logger.log(Level::Info, format!("{:?}: {}: {}", self.identity, self.context, message));
    }

    pub fn warn<T: Display>(&self, message: T) {
        self.logger.log(Level::Warn, format!("{:?}: {}: {}", self.identity, self.context, message));
    }

    pub fn incoming_event<T: Debug>(&self, event: T) {
        self.logger.log(Level::Info, format!("{:?}: {} RECV: {:?}", self.identity, self.context, &event));
    }

    pub fn outgoing_event<T: Debug>(&self, event: T) {
        self.logger.log(Level::Info, format!("{:?}: {} SEND: {:?}", self.identity, self.context, &event));
    }
}