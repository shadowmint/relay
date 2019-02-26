use ws::Handler;
use crate::infrastructure::relay_logger::RelayLogger;
use ws::Message;
use ws::CloseCode;
use ws;
use ws::Sender;
use rust_isolate::IsolateRuntimeRef;
use crate::events::master_event::MasterEvent;
use crate::events::client_event::ClientEvent;
use rust_isolate::IsolateChannel;
use crate::server::server_error::ServerError;
use crate::events::client_event::ClientExternalEvent;
use crate::events::master_event::MasterExternalEvent;
use crate::events::master_event::MasterExternalEvent::MasterDisconnected;
use crate::events::client_event::ClientExternalEvent::ClientDisconnected;
use std::error::Error;
use std::thread;

pub enum ServerEvent {
    Client(ClientEvent),
    Master(MasterEvent),
}

pub enum ServerConnectionState {
    None,
    Client(IsolateChannel<ClientEvent>),
    Master(IsolateChannel<MasterEvent>),
}

impl ServerConnectionState {
    pub fn is_none(&self) -> bool {
        match self {
            ServerConnectionState::None => true,
            _ => false
        }
    }
}

pub struct ServerConnection {
    state: ServerConnectionState,
    output: Option<Sender>,
    logger: RelayLogger,
    pub masters: IsolateRuntimeRef<MasterEvent>,
    pub clients: IsolateRuntimeRef<ClientEvent>,
}

impl ServerConnection {
    pub fn new(output: Option<Sender>, masters: IsolateRuntimeRef<MasterEvent>, clients: IsolateRuntimeRef<ClientEvent>, logger: RelayLogger) -> ServerConnection {
        ServerConnection {
            state: ServerConnectionState::None,
            output,
            masters,
            clients,
            logger,
        }
    }

    /// Try to guess the state from the message type
    fn pick_state_from(&mut self, message: &str) -> Result<(), ServerError> {
        match serde_json::from_str::<MasterExternalEvent>(message) {
            Ok(_) => {
                self.become_master()?;
                return Ok(());
            }
            Err(_) => {}
        }
        match serde_json::from_str::<ClientExternalEvent>(message) {
            Ok(_) => {
                self.become_client()?;
                return Ok(());
            }
            Err(_) => {}
        }
        self.logger.warn(format!("Invalid message: {}", message));
        Ok(())
    }

    /// Dispatch message based on type
    fn dispatch_message(&self, message: &str) -> Result<(), ServerError> {
        match &self.state {
            ServerConnectionState::Master(channel) => {
                let message = serde_json::from_str::<MasterExternalEvent>(message)?;
                channel.sender.send(MasterEvent::External(message))?;
            }
            ServerConnectionState::Client(channel) => {
                let message = serde_json::from_str::<ClientExternalEvent>(message)?;
                channel.sender.send(ClientEvent::External(message))?;
            }
            ServerConnectionState::None => {}
        }
        Ok(())
    }

    /// Become a master instance
    fn become_master(&mut self) -> Result<(), ServerError> {
        let channel = self.masters.spawn()?;
        self.spawn_master_reader(channel.clone().unwrap());
        self.state = ServerConnectionState::Master(channel);
        Ok(())
    }

    /// Become a client instance
    fn become_client(&mut self) -> Result<(), ServerError> {
        let channel = self.clients.spawn()?;
        self.spawn_client_reader(channel.clone().unwrap());
        self.state = ServerConnectionState::Client(channel);
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
}

impl Handler for ServerConnection {
    fn on_message(&mut self, msg: Message) -> ws::Result<()> {
        match msg {
            Message::Text(message) => {
                if self.state.is_none() {
                    self.pick_state_from(&message);
                }
                self.dispatch_message(&message);
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
            ServerConnectionState::Master(channel) => {
                self.send(channel, MasterEvent::External(MasterDisconnected { reason }));
            }
            ServerConnectionState::Client(channel) => {
                self.send(channel, ClientEvent::External(ClientDisconnected { reason }));
            }
            ServerConnectionState::None => {}
        }
    }
}