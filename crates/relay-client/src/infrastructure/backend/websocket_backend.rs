use crate::errors::relay_error::RelayError;
use crate::infrastructure::managed_connection::ManagedConnectionHandler;
use crate::infrastructure::relay_event::RelayEvent;
use crate::infrastructure::transaction_manager::TransactionManager;
use data_encoding::BASE64;
use futures::channel::oneshot;
use relay_auth::AuthRequest;
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
    connected: bool,
    resolver: Arc<Mutex<Option<oneshot::Sender<Result<Box<dyn ManagedConnectionHandler + Send + 'static>, RelayError>>>>>,
    transaction_manager: TransactionManager,
    channel: Option<crossbeam::Sender<RelayEvent>>,
    out: Option<ws::Sender>,
}

impl WebSocketBackend {
    pub async fn new(
        remote: &str,
        transaction_manager: TransactionManager,
        channel: crossbeam::Sender<RelayEvent>,
        auth: Result<AuthRequest, RelayError>,
    ) -> Result<Box<dyn ManagedConnectionHandler + Send + 'static>, RelayError> {
        let (resolve, promise) = oneshot::channel();
        let resolve_sharable = Arc::new(Mutex::new(Some(resolve)));
        println!("Make token");
        // Resolve auth token
        let token = WebSocketBackend::get_token(auth)?;

        // Spawn the websocket worker function
        let auth_uri = format!("{}/?token={}", remote, token);
        println!("spawn xxx");
        thread::spawn(move || {
            let err_reporter = resolve_sharable.clone();
            match connect(auth_uri, |out| {
                return WebSocketHandler {
                    connected: false,
                    transaction_manager: transaction_manager.clone(),
                    resolver: resolve_sharable.clone(),
                    channel: Some(channel.clone()),
                    out: Some(out),
                };
            }) {
                Err(_) => {
                    println!("Connect failed");
                    let failure = Err(RelayError::ConnectionFailed("Unable to connect to remote".to_string()));
                    let _ = WebSocketHandler::resolve(&err_reporter, failure);
                }
                Ok(r) => {
                    println!("Got ok from connect");
                }
            }
            println!("Done with spawn");
        });

        println!("Wait for promise");
        // Return a promise for the api
        return match promise.await? {
            Ok(v) => Ok(v),
            Err(e) => Err(RelayError::SyncError(format!("{}", e))),
        };
    }

    fn get_token(auth: Result<AuthRequest, RelayError>) -> Result<String, RelayError> {
        match auth {
            Ok(event) => {
                let as_string = serde_json::to_string(&event)?;
                let as_base64 = BASE64.encode(as_string.as_bytes());
                return Ok(as_base64);
            }
            Err(err) => Err(err),
        }
    }
}

impl ManagedConnectionHandler for WebSocketBackend {
    fn send(&self, event: RelayEvent) -> Result<(), ()> {
        let raw = match event {
            RelayEvent::Client(e) => serde_json::to_string(&e),
            RelayEvent::Master(e) => serde_json::to_string(&e),
        };
        match raw {
            Ok(data) => match self.out.send(data) {
                Ok(_) => Ok(()),
                Err(err) => {
                    println!("Error sending message: {}", err);
                    Err(())
                }
            },
            Err(e) => {
                println!("ERROR!: {}", e.description());
                Err(())
            }
        }
    }
}

impl WebSocketHandler {
    fn on_connected(&mut self) {
        let connected = Box::new(WebSocketBackend {
            out: self.out.as_ref().unwrap().clone(),
        }) as Box<dyn ManagedConnectionHandler + Send + 'static>;
        self.connected = true;
        match WebSocketHandler::resolve(&self.resolver, Ok(connected)) {
            Ok(_) => {}
            Err(_) => match &self.out {
                Some(out) => {
                    let _ = out.close(CloseCode::Abnormal);
                }
                None => {}
            },
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
        promise: &Arc<Mutex<Option<oneshot::Sender<Result<Box<dyn ManagedConnectionHandler + Send + 'static>, RelayError>>>>>,
        result: Result<Box<dyn ManagedConnectionHandler + Send + 'static>, RelayError>,
    ) -> Result<(), RelayError> {
        let mut promise_arc = promise.lock()?;
        match promise_arc.take() {
            Some(promise) => {
                println!("Send err");
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
    fn on_open(&mut self, _: ws::Handshake) -> ws::Result<()> {
        println!("On open");
        self.on_connected();
        Ok(())
    }

    fn on_message(&mut self, msg: ws::Message) -> ws::Result<()> {
        println!("On on_message");
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
                None => match self.channel.as_ref() {
                    Some(channel) => {
                        let _ = channel.send(e);
                    }
                    None => {}
                },
            },
            Err(e) => {
                println!("Discarded message: {:?}", e);
            }
        }
        Ok(())
    }

    fn on_close(&mut self, _: CloseCode, _: &str) {
        let _ = self.out.take().unwrap().shutdown();
        self.channel.take();
    }

    fn on_error(&mut self, err: ws::Error) {
        println!("On on_error");
        if !self.connected {
            let failure = Err(RelayError::ConnectionFailed(format!(
                "Unable to connect to remote: {:?}: {}",
                err.kind, err.details
            )));
            println!("??: {}", err);
            let _ = WebSocketHandler::resolve(&self.resolver, failure);
        }
        let _ = self.out.take().unwrap().shutdown();
        self.channel.take();
    }
}
