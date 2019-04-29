use crate::errors::relay_error::RelayError;
use crate::master::{Master, MasterOptions};
use crate::relay_serializer::RelaySerializer;
use crate::RelayEvent;
use futures::future::Either;
use futures::Future;
use relay_core::events::master_event::MasterExternalEvent;
use std::thread;
use uuid::Uuid;

pub enum MasterEvent<T> {
    Event { client_id: String, event: T },
    Meta(MasterExternalEvent),
}

pub struct MasterRelay<T> {
    inner: Master,
    channel: crossbeam::Receiver<MasterEvent<T>>,
    serializer: Box<RelaySerializer<T>>,
}

impl<T: Send + 'static> MasterRelay<T> {
    pub fn new(
        options: MasterOptions,
        serializer: impl RelaySerializer<T> + Send + Clone + 'static,
    ) -> impl Future<Item = MasterRelay<T>, Error = RelayError> {
        let (sx, rx) = crossbeam::unbounded();
        Master::new(options).then(|v| match v {
            Ok(master) => {
                let remote_reader = master.channel().clone();
                let remote_serializer = serializer.clone();
                thread::spawn(move || MasterRelay::<T>::event_loop(remote_reader, sx, remote_serializer));
                Ok(MasterRelay {
                    inner: master,
                    channel: rx,
                    serializer: Box::new(serializer),
                })
            }
            Err(e) => Err(e),
        })
    }

    pub fn channel(&self) -> &crossbeam::Receiver<MasterEvent<T>> {
        return &self.channel;
    }

    pub fn send(&self, client_id: &str, event: T) -> impl Future<Item = (), Error = RelayError> {
        match self.serializer.serialize(&event) {
            Ok(value) => {
                let outgoing_event = MasterExternalEvent::MessageToClient {
                    transaction_id: Uuid::new_v4().to_string(),
                    client_id: client_id.to_string(),
                    data: value,
                };
                Either::B(self.inner.send(outgoing_event))
            }
            Err(e) => Either::A(futures::failed(RelayError::from(e))),
        }
    }

    fn event_loop(reader: crossbeam::Receiver<MasterExternalEvent>, sender: crossbeam::Sender<MasterEvent<T>>, serializer: impl RelaySerializer<T>) {
        loop {
            match reader.recv() {
                Ok(v) => {
                    match v.clone() {
                        MasterExternalEvent::ClientJoined { client_id: _, name: _ } => {
                            let _ = sender.send(MasterEvent::Meta(v));
                        }
                        MasterExternalEvent::ClientDisconnected { client_id: _, reason: _ } => {
                            let _ = sender.send(MasterEvent::Meta(v));
                        }
                        MasterExternalEvent::MessageFromClient { client_id, data } => match serializer.deserialize(&data) {
                            Ok(value) => {
                                let outgoing_event = MasterEvent::Event { client_id, event: value };
                                let _ = sender.send(outgoing_event);
                            }
                            Err(e) => {
                                // Log error
                            }
                        },
                        _ => {
                            // Log error
                        }
                    }
                }
                Err(e) => {
                    break;
                }
            }
        }
    }
}
