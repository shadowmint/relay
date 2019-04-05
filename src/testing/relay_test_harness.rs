use relay_core::events::client_event::ClientEvent;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::model::client_metadata::ClientMetadata;
use relay_core::model::master_metadata::MasterMetadata;
use rust_isolate::IsolateChannel;
use crate::server::server_connection_factory::ServerConnectionFactory;
use crate::server::server_connection::ServerConnection;
use crate::ServerConfig;
use std::collections::HashMap;

/// This is common helper for running application state tests
pub struct RelayTestHarness {
    pub factory: ServerConnectionFactory,
    pub instance: Option<ServerConnection>,
}

impl RelayTestHarness {
    /// Create a new test harness
    pub fn new() -> RelayTestHarness {
        RelayTestHarness {
            factory: ServerConnectionFactory::new(ServerConfig {
                bind: "".to_string(),
                secrets: HashMap::new(),
            }).unwrap(),
            instance: None,
        }
    }

    /// Create a new session and join a set of peers to it
    pub fn create_session(&mut self, session_name: &str, peers: usize, max_peers: usize) -> (IsolateChannel<MasterEvent>, Vec<IsolateChannel<ClientEvent>>) {
        let mut service = self.factory.new_connection(None).unwrap();

        // Create a master instance
        let master = service.masters.spawn().unwrap();
        master.sender.send(MasterEvent::External(MasterExternalEvent::InitializeMaster {
            transaction_id: format!("Test"),
            metadata: MasterMetadata {
                max_clients: max_peers as u32,
                master_id: session_name.to_string(),
            },
        })).unwrap();

        // Validate session started ok
        match master.receiver.recv() {
            Ok(r) => {
                match r {
                    MasterEvent::External(er) => {
                        match er {
                            MasterExternalEvent::TransactionResult { transaction_id: _, success, error } => {
                                assert!(success);
                                assert!(error.is_none());
                            }
                            _ => unreachable!()
                        }
                    }
                    _ => unreachable!()
                }
            }
            Err(_) => unreachable!()
        };

        // Create a set of clients
        let mut clients = Vec::new();
        for i in 0..peers {

            // Initialize client
            let client = service.clients.spawn().unwrap();
            client.sender.send(ClientEvent::External(ClientExternalEvent::InitializeClient {
                transaction_id: format!("Test-{}", i),
                metadata: ClientMetadata {
                    name: format!("Player {}", i)
                },
            })).unwrap();

            // Check init passed
            match client.receiver.recv() {
                Ok(r) => {
                    match r {
                        ClientEvent::External(er) => {
                            match er {
                                ClientExternalEvent::TransactionResult { transaction_id: _, success, error: _ } => {
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
                session_id: session_name.to_string(),
            })).unwrap();

            // Check join passed
            match client.receiver.recv() {
                Ok(r) => {
                    match r {
                        ClientEvent::External(er) => {
                            match er {
                                ClientExternalEvent::TransactionResult { transaction_id: _, success, error } => {
                                    assert!(success);
                                    assert!(error.is_none());
                                }
                                _ => unreachable!()
                            }
                        }
                        _ => unreachable!()
                    }
                }
                Err(_) => unreachable!()
            }

            clients.push(client);
        }

        self.instance = Some(service);
        return (master, clients);
    }

    /// Complete this test run
    pub fn complete(self) {
        self.factory.registry.wait();
    }
}
