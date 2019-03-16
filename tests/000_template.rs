use relay::RelayTestHarness;
use relay_core::events::master_event::MasterControlEvent;
use relay_core::events::client_event::ClientControlEvent;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::client_event::ClientEvent;
use std::thread;
use std::time::Duration;

#[test]
pub fn main() {
    let mut harness = RelayTestHarness::new();
    let (master, clients) = harness.create_session("Hello World", 1, 2);

    // Wait for processing to finish
    thread::sleep(Duration::from_millis(100));

    // Halt everyone
    master.sender.send(MasterEvent::Control(MasterControlEvent::Halt)).unwrap();
    clients[0].sender.send(ClientEvent::Control(ClientControlEvent::Halt)).unwrap();

    harness.complete();
}