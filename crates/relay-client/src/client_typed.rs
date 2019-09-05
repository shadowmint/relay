use crate::client::Client;
use crate::errors::relay_error::RelayError;
use crate::infrastructure::relay_event::RelayEvent;
use crate::ClientOptions;
use futures::future::Either;
use futures::Future;
use relay_core::events::client_event::ClientExternalEvent;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::io::BufReader;
use std::thread;
use uuid::Uuid;

#[derive(Debug)]
pub enum ClientEvent<TEvent> {
    External(ClientExternalEvent),
    Internal(TEvent),
}

pub struct ClientTyped<TEvent> {
    client: Client,
    input: crossbeam::Receiver<ClientEvent<TEvent>>,
}

impl<TEvent: Send + Serialize + DeserializeOwned + Debug + 'static> ClientTyped<TEvent> {
    /// Create a new instance
    pub fn new(options: ClientOptions) -> impl Future<Item = ClientTyped<TEvent>, Error = RelayError> {
        Client::new(options).then(|r| {
            let client = r?;
            let (sx, rx) = crossbeam::unbounded();
            let reader = client.channel().clone();
            ClientTyped::<TEvent>::event_loop(reader, sx);
            Ok(ClientTyped { client: client, input: rx })
        })
    }

    /// Return a receiver channel for this connection
    pub fn channel(&self) -> crossbeam::Receiver<ClientEvent<TEvent>> {
        return self.input.clone();
    }

    /// Send an external event
    pub fn send(&self, event: ClientEvent<TEvent>) -> impl Future<Item = (), Error = RelayError> {
        match event {
            ClientEvent::External(ext) => Either::A(self.client.send(ext)),
            ClientEvent::Internal(event) => Either::B(self.send_to_master(event)),
        }
    }

    fn send_to_master(&self, event: TEvent) -> impl Future<Item = (), Error = RelayError> {
        let raw = match self.serialize(event) {
            Ok(r) => r,
            Err(e) => return Either::B(futures::failed(e)),
        };
        let event = ClientExternalEvent::MessageFromClient {
            transaction_id: Uuid::new_v4().to_string(),
            data: raw,
        };
        Either::A(self.client.send(event))
    }

    fn serialize(&self, event: TEvent) -> Result<String, RelayError> {
        let raw = serde_json::to_string(&event)?;
        Ok(raw)
    }

    fn event_loop(receiver: crossbeam::Receiver<RelayEvent>, sender: crossbeam::Sender<ClientEvent<TEvent>>) {
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok(relay_event) => {
                    match relay_event {
                        RelayEvent::Client(event) => {
                            let should_deserialize = match &event {
                                ClientExternalEvent::MessageToClient { data: _ } => true,
                                _ => false,
                            };
                            if should_deserialize {
                                match event {
                                    ClientExternalEvent::MessageToClient { data } => {
                                        match Self::deserialize(data) {
                                            Ok(internal_event) => {
                                                match sender.send(ClientEvent::Internal(internal_event)) {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        // TODO: Log it
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                // TODO: Log it
                                            }
                                        }
                                    }
                                    _ => {
                                        // TODO: Log it
                                    }
                                }
                            } else {
                                let _ = sender.send(ClientEvent::External(event));
                            }
                        }
                        _ => {}
                    }
                }
                Err(_) => {
                    break;
                }
            }
        });
    }

    fn deserialize(raw: String) -> Result<TEvent, RelayError> {
        Ok(serde_json::from_reader(BufReader::new(raw.as_bytes()))?)
    }
}

#[cfg(test)]
mod tests {
    use crate::infrastructure::testing::block_on_future;
    use crate::{AuthOptions, BackendType};
    use crate::{ClientOptions, ClientTyped};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "object_type")]
    pub enum TestEventType {
        EventOne,
        EventTwo,
    }

    #[test]
    fn test_create_client() {
        let _ = block_on_future(ClientTyped::<TestEventType>::new(ClientOptions {
            client_id: "Client".to_string(),
            session_id: "Master".to_string(),
            remote: "123".to_string(),
            backend: BackendType::Mock,
            auth: AuthOptions {
                key: "1234567890".to_string(),
                secret: "1234567890".to_string(),
                session_expires_secs: 1800,
            },
        }))
        .unwrap();
    }
}
