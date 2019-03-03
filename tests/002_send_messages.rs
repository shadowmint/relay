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
    let (master, clients) = harness.create_session("Hello World", 1, 2);

    // Read from the master and send a message to all clients
    match master.receiver.recv() {
        Ok(event) => {
            match event {
                MasterEvent::External(external) => {
                    match external {
                        MasterExternalEvent::ClientJoined { client_id: _, name } => {
                            assert_eq!(name, "Player 0")
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
        data: "Hello world".to_string(),
    })).unwrap();

    // Get a transaction result from sending the message
    match clients[0].receiver.recv() {
        Ok(event) => {
            match event {
                ClientEvent::External(external) => {
                    match external {
                        ClientExternalEvent::TransactionResult { transaction_id: _, success: _, error: _ } => {}
                        _ => unreachable!()
                    }
                }
                _ => unreachable!()
            }
        }
        _ => unreachable!()
    };

    // Read from the master and get the id of the sender
    let identity = match master.receiver.recv() {
        Ok(event) => {
            match event {
                MasterEvent::External(external) => {
                    match external {
                        MasterExternalEvent::MessageFromClient { client_id, data: _ } => {
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

    // Sleep for a bit to pretend the master was reading the message...
    thread::sleep(Duration::from_millis(100));

    // Send a response
    master.sender.send(MasterEvent::External(MasterExternalEvent::MessageToClient {
        client_id: identity,
        transaction_id: "1".to_string(),
        data: "Hello world back!".to_string(),
    })).unwrap();

    // Read from the clients
    match clients[0].receiver.recv() {
        Ok(event) => {
            match event {
                ClientEvent::External(external) => {
                    match external {
                        ClientExternalEvent::MessageToClient { data } => {
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