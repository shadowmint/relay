use relay::RelayTestHarness;
use relay_core::events::master_event::MasterControlEvent;
use relay_core::events::client_event::ClientControlEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::client_event::ClientEvent;
use std::thread;
use std::time::Duration;
use relay_core::model::master_metadata::MasterMetadata;
use relay_core::model::client_metadata::ClientMetadata;

#[test]
pub fn main() {
    let mut harness = RelayTestHarness::new();
    let (master, clients) = harness.create_session("Hello World", 2, 2);
    let mut service = harness.instance.as_mut().unwrap();

    // Wait for processing to finish
    thread::sleep(Duration::from_millis(100));

    // Initialize a client that can't connect because too many are connected
    let client = service.clients.spawn().unwrap();
    client.sender.send(ClientEvent::External(ClientExternalEvent::InitializeClient {
        transaction_id: format!("Test-NOPE"),
        metadata: ClientMetadata {
            name: format!("Player NOPE")
        },
    })).unwrap();

    // Check we got a valid response
    match client.receiver.recv() {
        Ok(r) => {
            match r {
                ClientEvent::External(er) => {
                    match er {
                        ClientExternalEvent::TransactionResult { transaction_id, success, error: _ } => {
                            assert!(success);
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        Err(_) => unreachable!()
    }

    // Now join the client to the given session id
    client.sender.send(ClientEvent::External(ClientExternalEvent::Join {
        transaction_id: format!("Test"),
        session_id: format!("Hello World"),
    })).unwrap();

    // Check we got a valid response
    match client.receiver.recv() {
        Ok(r) => {
            match r {
                ClientEvent::External(er) => {
                    match er {
                        ClientExternalEvent::TransactionResult { transaction_id, success, error } => {
                            assert!(!success);
                            assert!(error.is_some());
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        Err(_) => unreachable!()
    }

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