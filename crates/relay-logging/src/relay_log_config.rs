use base_logging::Level;

pub struct RelayLogConfig {
    pub level: Level
}

impl Default for RelayLogConfig {
    fn default() -> Self {
        RelayLogConfig {
            level: Level::Info
        }
    }
}