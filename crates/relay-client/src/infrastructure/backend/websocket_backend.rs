use crate::errors::relay_error::RelayError;
use crate::infrastructure::managed_connection::ManagedConnectionHandler;
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use futures::sync::oneshot;
use futures::Future;
use relay_auth::AuthEvent;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterExternalEvent;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use ws::{connect, CloseCode};

pub struct WebSocketBackend {
    out: ws::Sender,
}

struct WebSocketHandler {
    resolver: Arc<Mutex<Option<oneshot::Sender<Result<Box<dyn ManagedConnectionHandler + Send + 'static>, RelayError>>>>>,
    transaction_manager: TransactionManager,
    channel: crossbeam::Sender<RelayEvent>,
    out: ws::Sender,
}

impl WebSocketBackend {
    pub fn new(
        remote: &str,
        transaction_manager: TransactionManager,
        channel: crossbeam::Sender<RelayEvent>,
        auth: Result<AuthEvent, RelayError>,
    ) -> impl Future<Item = Box<dyn ManagedConnectionHandler + Send + 'static>, Error = RelayError> {
        let (resolve, promise) = oneshot::channel();
        let resolve_sharable = Arc::new(Mutex::new(Some(resolve)));
        let remote_owned = remote.to_string();

        // Spawn the websocket worker function
        thread::spawn(move || {
            let err_reporter = resolve_sharable.clone();
            if let Err(error) = connect(remote_owned, |out| {
                return WebSocketHandler {
                    transaction_manager: transaction_manager.clone(),
                    resolver: resolve_sharable.clone(),
                    channel: channel.clone(),
                    out,
                };
            }) {
                let failure = Err(RelayError::ConnectionFailed("Unable to connect to remote".to_string()));
                let _ = WebSocketHandler::resolve(&err_reporter, failure);
            }
        });

        // Return a promise for the api
        return promise.then(|r| match r {
            Ok(x) => match x {
                Ok(handler) => Ok(handler),
                Err(e) => Err(e),
            },
            Err(e) => Err(RelayError::SyncError(e.description().to_string())),
        });
    }
}

impl ManagedConnectionHandler for WebSocketBackend {
    fn send(&self, event: RelayEvent) {
        let raw = match event {
            RelayEvent::Auth(e) => serde_json::to_string(&e),
            RelayEvent::Client(e) => serde_json::to_string(&e),
            RelayEvent::Master(e) => serde_json::to_string(&e),
        };
        match raw {
            Ok(data) => {
                let _ = self.out.send(data);
            }
            Err(e) => {
                println!("ERROR!: {}", e.description());
            }
        }
    }
}

impl WebSocketHandler {
    fn on_connected(&mut self) {
        let connected = Box::new(WebSocketBackend { out: self.out.clone() }) as Box<ManagedConnectionHandler + Send + 'static>;
        match WebSocketHandler::resolve(&self.resolver, Ok(connected)) {
            Ok(_) => {}
            Err(e) => {
                let _ = self.out.close(CloseCode::Abnormal);
            }
        }
    }

    pub fn as_event(&self, raw: ws::Message) -> Result<RelayEvent, RelayError> {
        match raw {
            ws::Message::Text(raw_string) => {
                // Try read this as a master event.
                match serde_json::from_str::<MasterExternalEvent>(&raw_string) {
                    Ok(master_event) => Ok(RelayEvent::Master(master_event)),
                    Err(_) => {
                        // Fallback; attempt as a client event?
                        match serde_json::from_str::<ClientExternalEvent>(&raw_string) {
                            Ok(client_event) => Ok(RelayEvent::Client(client_event)),
                            Err(_) => Err(RelayError::InvalidEvent(format!("Unknown event: {}", raw_string))),
                        }
                    }
                }
            }
            ws::Message::Binary(_) => Err(RelayError::InvalidEvent(format!("Binary chunk not supported"))),
        }
    }

    pub fn resolve(
        promise: &Arc<Mutex<Option<oneshot::Sender<Result<Box<ManagedConnectionHandler + Send + 'static>, RelayError>>>>>,
        result: Result<Box<ManagedConnectionHandler + Send + 'static>, RelayError>,
    ) -> Result<(), RelayError> {
        let mut promise_arc = promise.lock()?;
        match promise_arc.take() {
            Some(promise) => {
                if promise.send(result).is_err() {
                    return Err(RelayError::SyncError(format!("Failed to dispatch promise result")));
                }
            }
            None => {
                return Err(RelayError::SyncError(format!("Unable to resolve already resolved connection handler")));
            }
        }
        Ok(())
    }
}

impl ws::Handler for WebSocketHandler {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        println!("Connection handler invoked on websocket!");
        self.on_connected();
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        let event = self.as_event(msg);
        match event {
            Ok(e) => match e.transaction_id() {
                Some(transaction_id) => {
                    let result = e.transaction_result();
                    match self.transaction_manager.resolve(&transaction_id, result) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Failed to exec relay: {:?}", e);
                        }
                    }
                }
                None => {
                    self.channel.send(e);
                }
            },
            Err(e) => {
                println!("Discarded message: {:?}", e);
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        println!("on_close in websocket handler: {:?}: {}", code, reason);
    }

    /// Called when an error occurs on the WebSocket.
    fn on_error(&mut self, err: ws::Error) {
        println!("Error in websocket handler: {:?}", err);
    }
}
