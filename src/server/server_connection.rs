use crate::server::server_error::ServerError;
use chrono::Utc;
use data_encoding::BASE64;
use relay_analytics::analytics::Analytics;
use relay_auth::AuthProvider;
use relay_auth::AuthResponse;
use relay_core::events::client_event::ClientControlEvent::ClientDisconnected;
use relay_core::events::client_event::ClientEvent;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterControlEvent::MasterDisconnected;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::model::external_error::{ErrorCode, ExternalError};
use relay_logging::RelayLogger;
use rust_isolate::IsolateChannel;
use rust_isolate::IsolateRuntimeRef;
use std::error::Error;
use std::str::from_utf8;
use std::{mem, thread};
use ws;
use ws::CloseCode;
use ws::Handler;
use ws::Message;
use ws::Sender;

#[derive(Clone)]
pub struct ServerSession {
    expires: i64,
}

pub enum ServerEvent {
    Client(ClientEvent),
    Master(MasterEvent),
}

pub enum ServerConnectionState {
    None,
    Authorized(ServerSession),
    Client {
        channel: IsolateChannel<ClientEvent>,
        session: ServerSession,
    },
    Master {
        channel: IsolateChannel<MasterEvent>,
        session: ServerSession,
    },
}

impl ServerConnectionState {
    pub fn is_unresolved(&self) -> bool {
        match self {
            ServerConnectionState::Authorized(_) => true,
            _ => false,
        }
    }

    /// Check if this request is authorized.
    /// Returns an (authorized, is_expired) tuple.
    pub fn is_authorized(&self) -> (bool, bool) {
        match self {
            ServerConnectionState::None => (false, false),
            ServerConnectionState::Authorized(session) => (true, self.is_expired(session.expires)),
            ServerConnectionState::Client {
                channel: _,
                session,
            } => (true, self.is_expired(session.expires)),
            ServerConnectionState::Master {
                channel: _,
                session,
            } => (true, self.is_expired(session.expires)),
        }
    }

    fn is_expired(&self, expiry: i64) -> bool {
        return Utc::now().timestamp() > expiry;
    }
}

pub struct ServerConnection {
    auth: AuthProvider,
    state: ServerConnectionState,
    output: Option<Sender>,
    logger: RelayLogger,
    analytics: Analytics,
    pub masters: IsolateRuntimeRef<MasterEvent>,
    pub clients: IsolateRuntimeRef<ClientEvent>,
}

impl ServerConnection {
    pub fn new(
        output: Option<Sender>,
        masters: IsolateRuntimeRef<MasterEvent>,
        clients: IsolateRuntimeRef<ClientEvent>,
        analytics: Analytics,
        logger: RelayLogger,
        auth: AuthProvider,
    ) -> ServerConnection {
        ServerConnection {
            state: ServerConnectionState::None,
            auth,
            analytics,
            output,
            masters,
            clients,
            logger,
        }
    }

    /// Try to guess the state from the message type
    fn pick_state_from(&mut self, message: &str) -> Result<(), ServerError> {
        let mut state = ServerConnectionState::None;
        mem::swap(&mut state, &mut self.state);

        match &state {
            ServerConnectionState::Authorized(session) => {
                match serde_json::from_str::<MasterExternalEvent>(message) {
                    Ok(_) => {
                        self.become_master(session.clone())?;
                        return Ok(());
                    }
                    Err(_) => {}
                }
                match serde_json::from_str::<ClientExternalEvent>(message) {
                    Ok(_) => {
                        self.become_client(session.clone())?;
                        return Ok(());
                    }
                    Err(_) => {}
                }
                self.logger.warn(format!("Invalid message: {}", message));
            }
            _ => {
                self.logger.warn(format!(
                    "Invalid state transition: source must be Authorized"
                ));
            }
        };

        // If we got here, return the original server state
        mem::swap(&mut state, &mut self.state);
        Ok(())
    }

    /// Dispatch message based on type
    fn dispatch_message(&self, message: &str) -> Result<(), ServerError> {
        match &self.state {
            ServerConnectionState::Master {
                channel,
                session: _,
            } => {
                let message = serde_json::from_str::<MasterExternalEvent>(message)?;
                channel.sender.send(MasterEvent::External(message))?;
            }
            ServerConnectionState::Client {
                channel,
                session: _,
            } => {
                let message = serde_json::from_str::<ClientExternalEvent>(message)?;
                channel.sender.send(ClientEvent::External(message))?;
            }
            ServerConnectionState::None => {}
            ServerConnectionState::Authorized(_) => {}
        }
        Ok(())
    }

    /// Become a master instance
    fn become_master(&mut self, session: ServerSession) -> Result<(), ServerError> {
        let channel = self.masters.spawn()?;
        self.spawn_master_reader(channel.clone());
        self.state = ServerConnectionState::Master { channel, session };
        self.analytics.track_event("master", 1);
        self.analytics.track_event("master_total", 1);
        Ok(())
    }

    /// Become a client instance
    fn become_client(&mut self, session: ServerSession) -> Result<(), ServerError> {
        let channel = self.clients.spawn()?;
        self.spawn_client_reader(channel.clone());
        self.state = ServerConnectionState::Client { channel, session };
        self.analytics.track_event("client", 1);
        self.analytics.track_event("client_total", 1);
        Ok(())
    }

    fn spawn_client_reader(&mut self, channel: IsolateChannel<ClientEvent>) {
        if self.output.is_none() {
            self.logger
                .warn("No output channel for connection, not delivering output events");
            return;
        }

        let read_channel = channel.clone();
        let read_logger = self.logger.clone();
        let output = self.output.take().unwrap();
        thread::spawn(move || {
            loop {
                match read_channel.receiver.recv() {
                    Ok(message) => match message {
                        ClientEvent::External(event) => match serde_json::to_string(&event) {
                            Ok(serialized_event) => match output.send(serialized_event) {
                                Ok(_) => {}
                                Err(e) => {
                                    read_logger.warn(format!(
                                        "Failed to send message: {}",
                                        e.description()
                                    ));
                                }
                            },
                            Err(e) => {
                                read_logger.warn(format!(
                                    "Failed to serialize message: {}",
                                    e.description()
                                ));
                            }
                        },
                        _ => {
                            read_logger.warn(format!("Discarded unknown message: {:?}", message));
                        }
                    },
                    Err(_) => {
                        // Channel went down, the connection is dead
                        break;
                    }
                }
            }
        });
    }

    fn spawn_master_reader(&mut self, channel: IsolateChannel<MasterEvent>) {
        if self.output.is_none() {
            self.logger
                .warn("No output channel for connection, not delivering output events");
            return;
        }

        let read_channel = channel.clone();
        let read_logger = self.logger.clone();
        let output = self.output.take().unwrap();
        thread::spawn(move || {
            loop {
                match read_channel.receiver.recv() {
                    Ok(message) => match message {
                        MasterEvent::External(event) => match serde_json::to_string(&event) {
                            Ok(serialized_event) => match output.send(serialized_event) {
                                Ok(_) => {}
                                Err(e) => {
                                    read_logger.warn(format!(
                                        "Failed to send message: {}",
                                        e.description()
                                    ));
                                }
                            },
                            Err(e) => {
                                read_logger.warn(format!(
                                    "Failed to serialize message: {}",
                                    e.description()
                                ));
                            }
                        },
                        _ => {
                            read_logger.warn(format!("Discarded unknown message: {:?}", message));
                        }
                    },
                    Err(_) => {
                        // Channel went down, the connection is dead
                        break;
                    }
                }
            }
        });
    }

    fn send<T: Send + 'static>(&self, channel: &IsolateChannel<T>, event: T) {
        match channel.sender.send(event) {
            Ok(_) => {}
            Err(e) => {
                self.logger
                    .warn(format!("Failed to send event: {}", e.description()));
            }
        };
    }

    fn try_authorize(&mut self, request: &str) -> AuthResponse {
        self.auth.authorize(request)
    }

    /// Halt this socket connection
    fn halt(&mut self) {
        match self.output.as_mut() {
            Some(ref m) => {
                match m.close_with_reason(CloseCode::Error, "Server closed connection") {
                    Ok(_) => {}
                    Err(err) => {
                        self.logger
                            .error(format!("Failed to close socket: {}", err));
                    }
                }
            }
            None => {}
        }
    }

    /// Require authorization to continue
    fn require_auth(&mut self, message: Option<&str>) -> Result<(), ExternalError> {
        let (authorized, auth_expired) = self.state.is_authorized();

        // You must authorize before you can do anything.
        if !authorized && message.is_some() {
            match self.try_authorize(message.as_ref().unwrap()) {
                AuthResponse::Passed { event, expires } => {
                    self.logger.info(format!("Authorization success"));
                    self.state = ServerConnectionState::Authorized(ServerSession { expires });
                    return Ok(());
                }
                AuthResponse::Failed(event) => {
                    self.logger
                        .warn(format!("Auth failed: {:?}: {:?}", &event, message));
                    self.halt();
                    return Err(ExternalError::from(ErrorCode::InvalidRequest));
                }
            }
        }

        // Check token expiry
        if auth_expired {
            self.logger.warn(format!("Auth token expired"));
            self.state = ServerConnectionState::None;
            self.halt();
            return Err(ExternalError::from(ErrorCode::InvalidRequest));
        }

        return Ok(());
    }

    /// Extract the auth token from the incoming request
    fn message_from(&self, resource: &str) -> Option<String> {
        let prefix = "/?token=";
        if !resource.starts_with(prefix) {
            self.logger.warn(format!("Invalid token: bad prefix"));
            return None;
        }
        let encoded: String = resource.chars().skip(prefix.len()).collect();
        return match BASE64.decode(&encoded.as_bytes()) {
            Ok(values) => match from_utf8(&values) {
                Ok(v) => Some(v.to_string()),
                Err(err) => {
                    self.logger.warn(format!("Invalid token: {}", err));
                    None
                }
            },
            Err(err) => {
                self.logger.warn(format!("Invalid token: {}", err));
                None
            }
        };
    }
}

impl Handler for ServerConnection {
    fn on_open(&mut self, shake: ws::Handshake) -> ws::Result<()> {
        match self.message_from(&shake.request.resource()) {
            Some(message) => match self.require_auth(Some(&message)) {
                Ok(_) => {}
                Err(err) => {
                    self.logger.warn(format!("Auth failed: {}", err));
                }
            },
            None => {
                self.logger.warn("Auth rejected: no token specified");
                self.halt();
            }
        }
        Ok(())
    }

    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Text(message) => {
                match self.require_auth(None) {
                    Ok(_) => {}
                    Err(err) => {
                        return Err(ws::Error::new(
                            ws::ErrorKind::Custom(Box::new(err)),
                            "Invalid request".to_string(),
                        ));
                    }
                }

                // Pick master / client mode
                if self.state.is_unresolved() {
                    match self.pick_state_from(&message) {
                        Ok(_) => {}
                        Err(e) => {
                            self.logger
                                .warn(format!("Failed to pick object state: {:?}: {}", e, message));
                        }
                    }
                }

                // Process real messages
                match self.dispatch_message(&message) {
                    Ok(_) => {}
                    Err(e) => {
                        self.logger
                            .warn(format!("Failed to dispatch message: {:?}: {}", e, message));
                    }
                }
            }
            _ => {
                self.logger.warn("Discarded binary chunk");
            }
        }
        Ok(())
    }

    fn on_close(&mut self, code: CloseCode, reason: &str) {
        let reason = format!("Connection closing due to ({:?}) {}", code, reason);
        match &self.state {
            ServerConnectionState::Master {
                channel,
                session: _,
            } => {
                self.send(channel, MasterEvent::Control(MasterDisconnected { reason }));
                self.analytics.track_event("master", -1);
            }
            ServerConnectionState::Client {
                channel,
                session: _,
            } => {
                self.send(channel, ClientEvent::Control(ClientDisconnected { reason }));
                self.analytics.track_event("client", -1);
            }
            ServerConnectionState::Authorized(_) => {}
            ServerConnectionState::None => {}
        }
    }
}
