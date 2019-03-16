pub mod isolates;
pub mod analytics;
pub mod analytics_error;
pub mod analytics_context;

pub(crate) const ANALYTICS: &'static str = "analytics";

pub use crate::isolates::analytics_service::AnalyticsService;
pub use crate::isolates::analytics_service::analytics_events::AnalyticsEventType;

#[cfg(test)]
mod tests {
    use rust_isolate::IsolateRegistry;
    use crate::AnalyticsService;
    use crate::analytics::Analytics;
    use futures::future::Future;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_spawn_analytics_isolate() {
        let mut registry = IsolateRegistry::new();
        let mut runtime = AnalyticsService::bind(&mut registry).unwrap();
        { let _ = runtime.spawn().unwrap(); }
        registry.wait();
    }

    #[test]
    fn test_create_analytics_api() {
        let mut registry = IsolateRegistry::new();
        AnalyticsService::bind(&mut registry).unwrap();
        { let _ = Analytics::new(registry.as_ref()); }
        registry.wait();
    }

    #[test]
    fn test_send_analytics_events() {
        let mut registry = IsolateRegistry::new();
        AnalyticsService::bind(&mut registry).unwrap();
        {
            let analytics = Analytics::new(registry.as_ref()).unwrap();
            for _i in 1..100 {
                analytics.track_event("test", 1);
                analytics.track_event("test2", 1);
            }
            for _i in 1..50 {
                analytics.track_event("test2", -1);
            }

            let labels = analytics.query_event_labels(None).wait().unwrap();
            let value1 = analytics.query_event("test").wait().unwrap();
            let value2 = analytics.query_event("test2").wait().unwrap();

            // event are processed in a queue, so wait for it to flush.
            thread::sleep(Duration::from_millis(100));

            assert_eq!(99, value1);
            assert_eq!(50, value2); // 99 - 49
            assert!(labels.iter().any(|i| i == "test"));
            assert!(labels.iter().any(|i| i == "test2"));
        }
        registry.wait();
    }

    #[test]
    fn test_filter_labels() {
        let mut registry = IsolateRegistry::new();
        AnalyticsService::bind(&mut registry).unwrap();
        {
            let analytics = Analytics::new(registry.as_ref()).unwrap();
            analytics.track_event("test", 1);
            analytics.track_event("test2", 1);
            analytics.track_event("one2", 1);
            analytics.track_event("one1", 1);
            analytics.track_event("one", 1);
            analytics.track_event("one", 1);
            analytics.track_event("1one2", 1);
            analytics.track_event("two", 1);

            let labels = analytics.query_event_labels(Some("t.*".to_string())).wait().unwrap();
            let labels2 = analytics.query_event_labels(Some(".*one.*".to_string())).wait().unwrap();
            let labels_all = analytics.query_event_labels(None).wait().unwrap();

            assert_eq!(3, labels.len());
            assert_eq!(4, labels2.len());
            assert_eq!(7, labels_all.len());
        }
        registry.wait();
    }
}
