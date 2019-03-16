use base_logging::{LogFormatter, Level};
use std::collections::HashMap;
use time::Tm;
use crate::relay_logger::{CONTEXT, CONTEXT_DEFAULT};

pub struct RelayLogFormat {
    default_props: HashMap<&'static str, &'static str>
}

impl RelayLogFormat {
    pub fn new() -> RelayLogFormat {
        let mut props = HashMap::new();
        props.insert(CONTEXT, CONTEXT_DEFAULT);
        RelayLogFormat {
            default_props: props
        }
    }

    fn combine(&self, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        let mut rtn = String::new();
        let props = properties.as_ref().unwrap_or(&self.default_props);
        let context = props.get(CONTEXT).unwrap();
        match message {
            Some(msg) => {
                rtn.push_str(&format!("[{}] {}", context, msg));
            }
            None => {}
        };
        match properties {
            Some(props) => {
                for (key, value) in props.iter() {
                    if *key != CONTEXT {
                        rtn.push_str(&format!(" {} {}", key, value));
                    }
                }
            }
            None => {}
        }
        return rtn;
    }
}

impl LogFormatter for RelayLogFormat {
    fn log_format(&self, level: Level, timestamp: Tm, message: Option<&str>, properties: Option<HashMap<&str, &str>>) -> String {
        let time_string = match time::strftime("%d/%b/%Y:%H:%M:%S %z", &timestamp) {
            Ok(i) => i,
            Err(_) => String::from("")
        };
        return format!("[{}] [{:?}] {}", time_string, level, self.combine(message, properties));
    }
}