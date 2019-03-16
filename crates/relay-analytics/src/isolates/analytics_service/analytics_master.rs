use std::sync::{Arc, Mutex};
use crate::analytics_error::AnalyticsError;
use crate::isolates::analytics_service::analytics_events::AnalyticsQueryType;
use futures::sync::oneshot::Sender;
use crate::isolates::analytics_service::analytics_events::AnalyticsQueryType::{AnalyticsQueryEvents, AnalyticsQueryLabels};
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use regex::Regex;
use std::error::Error;
use crate::analytics_context::AnalyticsContext;

pub struct AnalyticsMaster {
    context: Arc<Mutex<AnalyticsContext>>
}

impl AnalyticsMaster {
    pub fn new(context: Arc<Mutex<AnalyticsContext>>) -> AnalyticsMaster {
        AnalyticsMaster {
            context
        }
    }

    pub fn track_event(&mut self, key: &str, count: i32) -> Result<(), AnalyticsError> {
        let mut shared = self.context.lock()?;
        let value = match shared.data.entry(key.to_string()) {
            Entry::Occupied(o) => o.into_mut(),
            Entry::Vacant(v) => v.insert(0)
        };
        *value = *value + count;
        Ok(())
    }

    pub fn run_query(&self, query: AnalyticsQueryType) -> Result<(), AnalyticsError> {
        match query {
            AnalyticsQueryLabels(filter, promise) => {
                self.query_labels(filter, promise)?;
                Ok(())
            }
            AnalyticsQueryEvents(labels, promise) => {
                self.query_events(labels, promise)?;
                Ok(())
            }
        }
    }

    fn query_labels(&self, filter: Option<String>, promise: Sender<Result<Vec<String>, AnalyticsError>>) -> Result<(), AnalyticsError> {
        let shared = self.context.lock()?;
        let all_labels = match filter {
            Some(f) => {
                // Apply a regex filter
                match Regex::new(&f) {
                    Ok(rgx) => {
                        shared.data.keys().filter(|i| {
                            rgx.is_match(i)
                        }).map(|i| i.to_string()).collect()
                    }
                    Err(e) => {
                        match promise.send(Err(AnalyticsError::QueryError(format!("Invalid regex: {}: {}", &f, e.description())))) {
                            Ok(_) => {}
                            Err(_) => {
                                return Err(AnalyticsError::AsyncError(format!("Failed to send result")));
                            }
                        }
                        return Ok(());
                    }
                }
            }
            None => {
                // No filter, apply all
                shared.data.keys().map(|i| i.to_string()).collect()
            }
        };
        match promise.send(Ok(all_labels)) {
            Ok(_) => Ok(()),
            Err(_) => Err(AnalyticsError::AsyncError(format!("Failed to send result")))
        }
    }

    fn query_events(&self, labels: Vec<String>, promise: Sender<Result<HashMap<String, i32>, AnalyticsError>>) -> Result<(), AnalyticsError> {
        let mut results: HashMap<String, i32> = HashMap::new();
        {
            let shared = self.context.lock()?;
            labels.iter().for_each(|l| {
                if labels.contains(l) {
                    results.insert(l.to_string(), shared.data[l]);
                } else {
                    results.insert(l.to_string(), 0);
                }
            });
        }
        match promise.send(Ok(results)) {
            Ok(_) => Ok(()),
            Err(_) => Err(AnalyticsError::AsyncError(format!("Failed to send result")))
        }
    }
}