use relay::RelayTestHarness;
use relay_core::events::master_event::MasterControlEvent;
use relay_core::events::master_event::MasterEvent;
use std::thread;
use std::time::Duration;

#[test]
pub fn main() {
    let mut harness = RelayTestHarness::new();
    let (master, _clients) = harness.create_session("Hello World", 2, 2);

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