use crate::errors::relay_error::RelayError;
use crate::infrastructure::relay_event::RelayEvent;
use crate::master::Master;
use crate::MasterOptions;
use relay_core::events::master_event::MasterExternalEvent;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io::BufReader;
use std::thread;
use uuid::Uuid;

#[derive(Debug)]
pub enum MasterEvent<TEvent> {
    External(MasterExternalEvent),
    Internal { client_id: String, event: TEvent },
}

pub struct MasterTyped<TEvent> {
    master: Master,
    input: crossbeam::Receiver<MasterEvent<TEvent>>,
}

impl<TEvent: Send + Serialize + DeserializeOwned + 'static> MasterTyped<TEvent> {
    /// Create a new instance
    pub async fn new(options: MasterOptions) -> Result<MasterTyped<TEvent>, RelayError> {
        let master = Master::new(options).await?;
        let (sx, rx) = crossbeam::unbounded();
        let reader = master.channel().clone();
        MasterTyped::<TEvent>::event_loop(reader, sx);
        Ok(MasterTyped { master, input: rx })
    }

    /// Return a receiver channel for this connection
    pub fn channel(&self) -> crossbeam::Receiver<MasterEvent<TEvent>> {
        return self.input.clone();
    }

    /// Send an external event
    pub async fn send(&self, event: MasterEvent<TEvent>) -> Result<(), RelayError> {
        match event {
            MasterEvent::External(ext) => self.master.send(ext).await,
            MasterEvent::Internal { client_id, event } => self.send_to_client(client_id, event).await,
        }
    }

    async fn send_to_client(&self, client_id: String, event: TEvent) -> Result<(), RelayError> {
        let raw = match self.serialize(event) {
            Ok(r) => r,
            Err(e) => return Err(e),
        };
        let event = MasterExternalEvent::MessageToClient {
            client_id,
            transaction_id: Uuid::new_v4().to_string(),
            data: raw,
        };
        self.master.send(event).await
    }

    fn serialize(&self, event: TEvent) -> Result<String, RelayError> {
        Ok(serde_json::to_string(&event)?)
    }

    fn event_loop(receiver: crossbeam::Receiver<RelayEvent>, sender: crossbeam::Sender<MasterEvent<TEvent>>) {
        thread::spawn(move || loop {
            match receiver.recv() {
                Ok(relay_event) => {
                    match relay_event {
                        RelayEvent::Master(event) => {
                            let should_deserialize = match &event {
                                MasterExternalEvent::MessageFromClient { client_id: _, data: _ } => true,
                                _ => false,
                            };
                            if should_deserialize {
                                match event {
                                    MasterExternalEvent::MessageFromClient { client_id, data } => {
                                        match Self::deserialize(data) {
                                            Ok(internal_event) => {
                                                match sender.send(MasterEvent::Internal {
                                                    client_id,
                                                    event: internal_event,
                                                }) {
                                                    Ok(_) => {}
                                                    Err(_) => {
                                                        // TODO: Log it
                                                    }
                                                };
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
                                let _ = sender.send(MasterEvent::External(event));
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
    use crate::{MasterOptions, MasterTyped};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    #[serde(tag = "object_type")]
    pub enum TestEventType {
        EventOne,
        EventTwo,
    }

    #[test]
    fn test_create_master() {
        let _ = block_on_future(MasterTyped::<TestEventType>::new(MasterOptions {
            master_id: "Master".to_string(),
            max_clients: 10,
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
