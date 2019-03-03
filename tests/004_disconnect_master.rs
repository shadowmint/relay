use relay::RelayTestHarness;
use relay::events::master_event::MasterControlEvent;
use relay::events::client_event::ClientControlEvent;
use relay::events::master_event::MasterExternalEvent;
use relay::events::client_event::ClientExternalEvent;
use relay::events::master_event::MasterEvent;
use relay::events::client_event::ClientEvent;
use std::thread;
use std::time::Duration;
use relay::model::master_metadata::MasterMetadata;
use relay::model::client_metadata::ClientMetadata;

#[test]
pub fn main() {
    let mut harness = RelayTestHarness::new();
    let (master, clients) = harness.create_session("Hello World", 2, 2);

    // Wait for processing to finish
    thread::sleep(Duration::from_millis(100));

    // Halt master
    master.sender.send(MasterEvent::Control(MasterControlEvent::MasterDisconnected {
        reason: format!("Test")
    })).unwrap();

    // Wait for processing to finish
    thread::sleep(Duration::from_millis(100));

    harness.complete();
}