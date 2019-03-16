use base_logging::Loggable;
use std::collections::HashMap;
use crate::relay_logger::CONTEXT;

pub struct RelayLogEntry<'a> {
    pub context: &'a str,
    pub message: &'a Loggable,
}

impl<'a> RelayLogEntry<'a> {
    pub fn new(context: &'a str, message: &'a Loggable) -> RelayLogEntry<'a> {
        return RelayLogEntry {
            context,
            message,
        };
    }
}

impl<'a> Loggable for RelayLogEntry<'a> {
    fn log_message(&self) -> Option<&str> {
        self.message.log_message()
    }

    fn log_properties(&self) -> Option<HashMap<&str, &str>> {
        match self.message.log_properties() {
            Some(mut s) => {
                s.insert(CONTEXT, self.context);
                return Some(s);
            }
            None => {
                let mut instance = HashMap::new();
                instance.insert(CONTEXT, self.context);
                return Some(instance);
            }
        }
    }
}