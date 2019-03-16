use futures::sync::oneshot::Sender;
use std::collections::HashMap;
use crate::analytics_error::AnalyticsError;

pub enum AnalyticsEventType {
    AnalyticsEvent(String, i32),
    AnalyticsQuery(AnalyticsQueryType),
}

pub enum AnalyticsQueryType {
    /// (Regex filter, promise)
    AnalyticsQueryLabels(Option<String>, Sender<Result<Vec<String>, AnalyticsError>>),

    /// (Label list, promise)
    AnalyticsQueryEvents(Vec<String>, Sender<Result<HashMap<String, i32>, AnalyticsError>>),
}