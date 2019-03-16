use std::collections::HashMap;

pub struct AnalyticsContext {
    pub data: HashMap<String, i32>
}

impl AnalyticsContext {
    pub fn new() -> AnalyticsContext {
        AnalyticsContext {
            data: HashMap::new()
        }
    }
}
