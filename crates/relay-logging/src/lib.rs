mod relay_log_entry;
mod relay_log_config;
mod relay_log_format;
mod relay_event_logger;
mod relay_logger;

pub use relay_logger::RelayLogger;
pub use relay_event_logger::RelayEventLogger;

#[cfg(test)]
mod tests {
    use crate::{RelayLogger, RelayEventLogger};

    #[test]
    fn test_logger() {
        let logger = RelayLogger::new("Test");
        logger.info("Hello world");
    }

    #[test]
    fn test_event_logger() {
        let logger = RelayEventLogger::new("12343-23123", "MASTER");
        logger.info("Spawn master");
        logger.incoming_event("Some event...");
    }
}
