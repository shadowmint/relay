use crate::analytics_error::AnalyticsError;
use rust_isolate::IsolateRegistryRef;
use crate::{AnalyticsEventType, ANALYTICS};
use crate::AnalyticsEventType::AnalyticsEvent;
use rust_isolate::IsolateChannel;
use futures::Future;
use futures::sync::oneshot;
use crate::isolates::analytics_service::analytics_events::AnalyticsQueryType::{AnalyticsQueryLabels, AnalyticsQueryEvents};
use crate::isolates::analytics_service::analytics_events::AnalyticsEventType::AnalyticsQuery;
use relay_logging::RelayLogger;
use futures::future::Either;
use std::collections::HashMap;

pub struct Analytics {
    channel: IsolateChannel<AnalyticsEventType>,
    logger: RelayLogger,
}

impl Analytics {
    pub fn new(registry: IsolateRegistryRef) -> Result<Analytics, AnalyticsError> {
        let mut runtime = registry.find::<AnalyticsEventType>(ANALYTICS)?;
        let channel = runtime.spawn()?;
        Ok(Analytics::from(channel))
    }

    pub fn track_event(&self, label: &str, delta: i32) {
        match self.channel.sender.send(AnalyticsEvent(label.to_string(), delta)) {
            Ok(_) => {}
            Err(e) => {
                self.logger.error(format!("failed to dispatch log entry: {}", e.to_string()))
            }
        }
    }

    /// If you provide a filter, it is treated as a regex on the label set.
    pub fn query_event_labels(&self, filter: Option<String>) -> impl Future<Item=Vec<String>, Error=AnalyticsError> {
        let (sx, rx) = oneshot::channel();
        match self.channel.sender.send(AnalyticsQuery(AnalyticsQueryLabels(filter, sx))) {
            Ok(_) => Either::A(rx.then(|r| {
                match r {
                    Ok(qr) => {
                        match qr {
                            Ok(result) => Ok(result),
                            Err(e) => Err(e)
                        }
                    }
                    Err(e) => Err(AnalyticsError::from(e))
                }
            })),
            Err(e) => Either::B(futures::failed(AnalyticsError::from(e)))
        }
    }

    pub fn query_event(&self, label: &str) -> impl Future<Item=i32, Error=AnalyticsError> {
        let label_str = label.to_string();
        let (sx, rx) = oneshot::channel();
        match self.channel.sender.send(AnalyticsQuery(AnalyticsQueryEvents(vec!(label_str.clone()), sx))) {
            Ok(_) => {
                Either::A(rx.then(move |r| {
                    match r {
                        Ok(qr) => {
                            match qr {
                                Ok(results) => Ok(*results.get(&label_str).unwrap_or(&-1)),
                                Err(err) => Err(AnalyticsError::from(err))
                            }
                        }
                        Err(e) => Err(AnalyticsError::from(e))
                    }
                }))
            }
            Err(e) => Either::B(futures::failed(AnalyticsError::from(e)))
        }
    }

    pub fn query_events<'a>(&self, labels: impl Iterator<Item=&'a str>) -> impl Future<Item=HashMap<String, i32>, Error=AnalyticsError> {
        let (sx, rx) = oneshot::channel();
        let labels_vec = labels.map(|i| i.to_string()).collect();
        match self.channel.sender.send(AnalyticsQuery(AnalyticsQueryEvents(labels_vec, sx))) {
            Ok(_) => Either::A(rx.then(|r| {
                match r {
                    Ok(qr) => {
                        match qr {
                            Ok(value) => Ok(value),
                            Err(e) => Err(e),
                        }
                    }
                    Err(e) => Err(AnalyticsError::from(e))
                }
            })),
            Err(e) => Either::B(futures::failed(AnalyticsError::from(e)))
        }
    }
}

impl From<IsolateChannel<AnalyticsEventType>> for Analytics {
    fn from(channel: IsolateChannel<AnalyticsEventType>) -> Self {
        Analytics {
            channel,
            logger: RelayLogger::new(ANALYTICS),
        }
    }
}