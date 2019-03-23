use relay::RelayTestHarness;
use relay_core::events::client_event::ClientControlEvent;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::client_event::ClientEvent;
use std::thread;
use std::time::Duration;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::events::master_event::MasterControlEvent;

#[test]
pub fn main() {
    let mut harness = RelayTestHarness::new();
    let (master, clients) = harness.create_session("Hello World", 1, 2);

    // Read a join event
    let client_id = match master.receiver.recv() {
        Ok(event) => {
            match event {
                MasterEvent::External(external) => {
                    match external {
                        MasterExternalEvent::ClientJoined { client_id, name } => {
                            assert_eq!(name, "Player 0");
                            client_id
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };

    // Send a message from a master to the client
    master.sender.send(MasterEvent::External(MasterExternalEvent::MessageToClient {
        client_id,
        transaction_id: "1".to_string(),
        data: "Hello world".to_string(),
    })).unwrap();

    // Read from the clients
    match clients[0].receiver.recv() {
        Ok(event) => {
            match event {
                ClientEvent::External(external) => {
                    match external {
                        ClientExternalEvent::MessageToClient { data } => {
                            assert_eq!(data, "Hello world");
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };

    // Read the transaction response from the master
    match master.receiver.recv() {
        Ok(event) => {
            match event {
                MasterEvent::External(external) => {
                    match external {
                        MasterExternalEvent::TransactionResult { transaction_id, success, error } => {
                            assert!(success);
                            assert!(error.is_none());
                            assert_eq!(transaction_id, "1");
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };

    // Wait for processing to finish
    thread::sleep(Duration::from_millis(100));

    // Halt everyone
    master.sender.send(MasterEvent::Control(MasterControlEvent::Halt)).unwrap();
    clients[0].sender.send(ClientEvent::Control(ClientControlEvent::Halt)).unwrap();

    harness.complete();
}