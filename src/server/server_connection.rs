use ws::{Handler, Handshake};
use ws::Message;
use ws::CloseCode;
use ws;
use ws::Sender;
use rust_isolate::IsolateRuntimeRef;
use relay_core::events::master_event::MasterEvent;
use relay_core::events::client_event::ClientEvent;
use rust_isolate::IsolateChannel;
use crate::server::server_error::ServerError;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::events::master_event::MasterControlEvent::MasterDisconnected;
use relay_core::events::client_event::ClientControlEvent::ClientDisconnected;
use std::error::Error;
use std::thread;
use relay_logging::RelayLogger;
use relay_analytics::analytics::Analytics;
use relay_auth::AuthProvider;
use chrono::Utc;
use core::borrow::Borrow;
use relay_auth::AuthResponse;

#[derive(Clone)]
pub struct ServerSession {
    expires: i64
}

pub enum ServerEvent {
    Client(ClientEvent),
    Master(MasterEvent),
}

pub enum ServerConnectionState {
    None,
    Authorized(ServerSession),
    Client { channel: IsolateChannel<ClientEvent>, session: ServerSession },
    Master { channel: IsolateChannel<MasterEvent>, session: ServerSession },
}

impl ServerConnectionState {
    pub fn is_none(&self) -> bool {
        match self {
            ServerConnectionState::None => true,
            _ => false
        }
    }

    /// Check if this request is authorized.
    /// Returns an (authorized, is_expired) tuple.
    pub fn is_authorized(&self) -> (bool, bool) {
        match self {
            ServerConnectionState::None => (false, false),
            ServerConnectionState::Authorized(session) => {
                (true, self.is_expired(session.expires))
            }
            ServerConnectionState::Client { channel: _, session } => {
                (true, self.is_expired(session.expires))
            }
            ServerConnectionState::Master { channel: _, session } => {
                (true, self.is_expired(session.expires))
            }
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
    pub fn new(output: Option<Sender>, masters: IsolateRuntimeRef<MasterEvent>,
               clients: IsolateRuntimeRef<ClientEvent>, analytics: Analytics,
               logger: RelayLogger, auth: AuthProvider) -> ServerConnection {
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
        match &self.state {
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
                Ok(())
            }
            _ => {
                self.logger.warn(format!("Invalid state transition: source must be Authorized"));
                Ok(())
            }
        }
    }

    /// Dispatch message based on type
    fn dispatch_message(&self, message: &str) -> Result<(), ServerError> {
        match &self.state {
            ServerConnectionState::Master { channel, session: _ } => {
                let message = serde_json::from_str::<MasterExternalEvent>(message)?;
                channel.sender.send(MasterEvent::External(message))?;
            }
            ServerConnectionState::Client { channel, session: _ } => {
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
        self.spawn_master_reader(channel.clone().unwrap());
        self.state = ServerConnectionState::Master { channel, session };
        self.analytics.track_event("master", 1);
        self.analytics.track_event("master_total", 1);
        Ok(())
    }

    /// Become a client instance
    fn become_client(&mut self, session: ServerSession) -> Result<(), ServerError> {
        let channel = self.clients.spawn()?;
        self.spawn_client_reader(channel.clone().unwrap());
        self.state = ServerConnectionState::Client { channel, session };
        self.analytics.track_event("client", 1);
        self.analytics.track_event("client_total", 1);
        Ok(())
    }

    fn spawn_client_reader(&mut self, channel: IsolateChannel<ClientEvent>) {
        if self.output.is_none() {
            self.logger.warn("No output channel for connection, not delivering output events");
            return;
        }

        let read_channel = channel.clone().unwrap();
        let read_logger = self.logger.clone();
        let output = self.output.take().unwrap();
        thread::spawn(move || {
            loop {
                match read_channel.receiver.recv() {
                    Ok(message) => {
                        match message {
                            ClientEvent::External(event) => {
                                match serde_json::to_string(&event) {
                                    Ok(serialized_event) => {
                                        match output.send(serialized_event) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                read_logger.warn(format!("Failed to send message: {}", e.description()));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        read_logger.warn(format!("Failed to serialize message: {}", e.description()));
                                    }
                                }
                            }
                            _ => {
                                read_logger.warn(format!("Discarded unknown message: {:?}", message));
                            }
                        }
                    }
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
            self.logger.warn("No output channel for connection, not delivering output events");
            return;
        }

        let read_channel = channel.clone().unwrap();
        let read_logger = self.logger.clone();
        let output = self.output.take().unwrap();
        thread::spawn(move || {
            loop {
                match read_channel.receiver.recv() {
                    Ok(message) => {
                        match message {
                            MasterEvent::External(event) => {
                                match serde_json::to_string(&event) {
                                    Ok(serialized_event) => {
                                        match output.send(serialized_event) {
                                            Ok(_) => {}
                                            Err(e) => {
                                                read_logger.warn(format!("Failed to send message: {}", e.description()));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        read_logger.warn(format!("Failed to serialize message: {}", e.description()));
                                    }
                                }
                            }
                            _ => {
                                read_logger.warn(format!("Discarded unknown message: {:?}", message));
                            }
                        }
                    }
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
                self.logger.warn(format!("Failed to send event: {}", e.description()));
            }
        };
    }

    fn try_authorize(&mut self, request: &str) -> AuthResponse {
        self.auth.authorize(request)
    }

    /// Halt this socket connection
    fn halt(&mut self) -> ws::Result<()> {
        match self.output.as_mut() {
            Some(ref m) => {
                m.close(CloseCode::Normal)
            }
            None => {
                Ok(())
            }
        }
    }

    /// Require authorization to continue
    fn require_auth(&mut self, message: &str) -> ws::Result<()> {
        let (authorized, auth_expired) = self.state.is_authorized();

        // You must authorize before you can do anything.
        if !authorized {
            match self.try_authorize(&message) {
                AuthResponse::Passed { event, expires } => {
                    self.state = ServerConnectionState::Authorized(ServerSession { expires });
                    match &self.output {
                        Some(out) => {
                            match serde_json::to_string(&event) {
                                Ok(raw) => {
                                    let _ = out.send(raw);
                                }
                                Err(_) => {}
                            }
                        }
                        None => {}
                    }
                }
                AuthResponse::Failed(event) => {
                    self.logger.warn(format!("Auth failed: {:?}: {}", &event, message));
                    match &self.output {
                        Some(out) => {
                            match serde_json::to_string(&event) {
                                Ok(raw) => {
                                    let _ = out.send(raw);
                                }
                                Err(_) => {}
                            }
                        }
                        None => {}
                    }
                    return self.halt();
                }
            }
        }

        // Check token expiry
        if auth_expired {
            self.logger.warn(format!("Auth token expired: Discarded request: {}", message));
            self.state = ServerConnectionState::None;
            return self.halt();
        }

        Ok(())
    }
}

impl Handler for ServerConnection {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Text(message) => {
                let auth_result = self.require_auth(&message);
                if auth_result.is_err() {
                    return auth_result;
                }

                // Pick master / client mode
                if self.state.is_none() {
                    match self.pick_state_from(&message) {
                        Ok(_) => {}
                        Err(e) => {
                            self.logger.warn(format!("Failed to pick object state: {:?}: {}", e, message));
                        }
                    }
                }

                // Process real messages
                match self.dispatch_message(&message) {
                    Ok(_) => {}
                    Err(e) => {
                        self.logger.warn(format!("Failed to dispatch message: {:?}: {}", e, message));
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
            ServerConnectionState::Master { channel, session: _ } => {
                self.send(channel, MasterEvent::Control(MasterDisconnected { reason }));
                self.analytics.track_event("master", -1);
            }
            ServerConnectionState::Client { channel, session: _ } => {
                self.send(channel, ClientEvent::Control(ClientDisconnected { reason }));
                self.analytics.track_event("client", -1);
            }
            ServerConnectionState::Authorized(_) => {}
            ServerConnectionState::None => {}
        }
    }
}