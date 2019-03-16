use std::fmt::Debug;
use std::fmt::Display;
use crate::RelayLogger;

const EVENT_LOGGER_CONTEXT: &'static str = "EventLogger";

#[derive(Clone)]
pub struct RelayEventLogger {
    context: String,
    identity: String,
    logger: RelayLogger,
}

impl RelayEventLogger {
    pub fn new(identity: &str, context: &str) -> RelayEventLogger {
        return RelayEventLogger {
            context: context.to_string(),
            identity: identity.to_string(),
            logger: RelayLogger::new(EVENT_LOGGER_CONTEXT),
        };
    }

    pub fn info<T: Display>(&self, message: T) {
        self.logger.info(format!("{}: {}: {}", self.identity, self.context, message));
    }

    pub fn warn<T: Display>(&self, message: T) {
        self.logger.warn(format!("{}: {}: {}", self.identity, self.context, message));
    }

    pub fn incoming_event<T: Debug>(&self, event: T) {
        self.logger.info(format!("{}: {} RECV: {:?}", self.identity, self.context, &event));
    }

    pub fn outgoing_event<T: Debug>(&self, event: T) {
        self.logger.info(format!("{}: {} SEND: {:?}", self.identity, self.context, &event));
    }
}