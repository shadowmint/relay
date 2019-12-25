pub mod analytics_events;
pub mod analytics_master;

use rust_isolate::{Isolate, IsolateIdentity, IsolateChannel, IsolateRegistry, IsolateRegistryError, IsolateRuntimeRef};
use crate::isolates::analytics_service::analytics_events::AnalyticsEventType;
use crate::ANALYTICS;
use crate::isolates::analytics_service::analytics_master::AnalyticsMaster;
use crate::analytics_error::AnalyticsError;
use relay_logging::RelayLogger;
use crate::isolates::analytics_service::analytics_events::AnalyticsEventType::{AnalyticsQuery, AnalyticsEvent};
use crate::analytics_context::AnalyticsContext;
use std::sync::{Mutex, Arc};

pub struct AnalyticsService {
    context: Arc<Mutex<AnalyticsContext>>
}

impl AnalyticsService {
    /// Create a new instance
    pub fn new(context: AnalyticsContext) -> AnalyticsService {
        AnalyticsService {
            context: Arc::new(Mutex::new(context))
        }
    }

    /// Register the analytics isolate with a registry
    pub fn bind(registry: &mut IsolateRegistry) -> Result<IsolateRuntimeRef<AnalyticsEventType>, IsolateRegistryError> {
        let context = AnalyticsContext::new();
        registry.bind(ANALYTICS, AnalyticsService::new(context))
    }
}

impl AnalyticsService {
    fn event_loop(channel: &IsolateChannel<AnalyticsEventType>, context: &Arc<Mutex<AnalyticsContext>>) {
        let logger = RelayLogger::new(ANALYTICS);
        match AnalyticsService::event_loop_checked(channel, context.clone()) {
            Ok(_) => {}
            Err(e) => {
                logger.error(e);
            }
        }
    }

    fn event_loop_checked(channel: &IsolateChannel<AnalyticsEventType>, context: Arc<Mutex<AnalyticsContext>>) -> Result<(), AnalyticsError> {
        let mut master = AnalyticsMaster::new(context);
        loop {
            match channel.receiver.recv() {
                Ok(record) => {
                    match record {
                        AnalyticsEvent(key, count) => {
                            master.track_event(&key, count)?;
                        }
                        AnalyticsQuery(query) => {
                            master.run_query(query)?;
                        }
                    }
                }
                Err(_) => {
                    // Disconnect
                    return Ok(());
                }
            }
        }
    }
}

impl Isolate<AnalyticsEventType> for AnalyticsService {
    fn spawn(&self, _: IsolateIdentity, channel: IsolateChannel<AnalyticsEventType>) -> Box<dyn FnMut() + Send + 'static> {
        let context = self.context.clone();
        Box::new(move || {
            AnalyticsService::event_loop(&channel, &context);
        })
    }
}