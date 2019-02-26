use relay::RelayTestHarness;
use relay::events::client_event::ClientControlEvent;
use relay::events::client_event::ClientExternalEvent;
use relay::events::master_event::MasterEvent;
use relay::events::client_event::ClientEvent;
use std::thread;
use std::time::Duration;
use relay::model::master_metadata::MasterMetadata;
use relay::model::client_metadata::ClientMetadata;
use relay::events::master_event::MasterExternalEvent;
use relay::events::master_event::MasterControlEvent;

#[test]
pub fn main() {
    let mut harness = RelayTestHarness::new();
    let (master, clients) = harness.create_game("Hello World", 1, 2);

    // Wait for the client to join
    match clients[0].receiver.recv() {
        Ok(event) => {
            match event {
                ClientEvent::External(external) => {
                    match external {
                        ClientExternalEvent::JoinResponse { success, error } => {
                            assert!(success);
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };

    // Send a message from a client to the master
    clients[0].sender.send(ClientEvent::External(ClientExternalEvent::MessageFromClient {
        transaction_id: "1".to_string(),
        format: "TEXT".to_string(),
        data: "Hello world".to_string(),
    }));

    // Read from the master and send a message to all clients
    let identity = match master.receiver.recv() {
        Ok(event) => {
            match event {
                MasterEvent::External(external) => {
                    match external {
                        MasterExternalEvent::ClientJoined { client, name } => {
                            client
                        }
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };

    // Sleep for a bit to pretend the master was reading the message...
    thread::sleep(Duration::from_millis(100));

    // Send a response
    master.sender.send(MasterEvent::External(MasterExternalEvent::MessageToClient {
        client: identity,
        transaction_id: "1".to_string(),
        format: "TEXT".to_string(),
        data: "Hello world back!".to_string(),
    }));

    // Read from the clients
    match clients[0].receiver.recv() {
        Ok(event) => {
            match event {
                ClientEvent::External(external) => {
                    match external {
                        ClientExternalEvent::MessageToClient { transaction_id, format, data } => {
                            assert_eq!(transaction_id, "1");
                            assert_eq!(format, "TEXT");
                            assert_eq!(data, "Hello world back!");
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