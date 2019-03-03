mod client_state;

use rust_isolate::Isolate;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateChannel;
use crate::events::client_event::ClientEvent;
use crate::infrastructure::relay_logger::RelayLogger;
use crate::infrastructure::services::SessionManager;
use crate::isolates::client::client_state::ClientState;
use crate::events::client_event::ClientExternalEvent;
use crate::events::client_event::ClientControlEvent;
use crate::isolates::client::ClientEventDispatch::DispatchExternal;
use crate::events::client_event::ClientInternalEvent;
use crate::events::master_event::MasterInternalEvent;
use crate::events::master_event::MasterEvent;
use crate::CLIENT;
use std::error::Error;
use crate::isolates::client::ClientEventDispatch::DispatchInternal;

#[derive(Debug)]
pub enum ClientEventDispatch {
    DispatchNone,
    DispatchExternal(ClientExternalEvent),
    DispatchInternal(MasterInternalEvent),
}

pub struct ClientIsolate {
    logger: RelayLogger,
    external: Option<IsolateChannel<ClientEvent>>,
    state: ClientState,
}

impl ClientIsolate {
    pub fn new(manager: SessionManager) -> ClientIsolate {
        ClientIsolate {
            state: ClientState::new(IsolateIdentity::new(), manager),
            logger: RelayLogger::new(None, CLIENT),
            external: None,
        }
    }

    pub fn instance(&self, identity: IsolateIdentity, channel: &IsolateChannel<ClientEvent>) -> ClientIsolate {
        ClientIsolate {
            state: self.state.instance(identity),
            logger: RelayLogger::new(Some(identity), CLIENT),
            external: Some(channel.clone().unwrap()),
        }
    }

    pub fn dispatch(&mut self, event: ClientEvent) -> Result<(), ()> {
        self.logger.incoming_event(&event);
        match event {
            ClientEvent::External(e) => {
                match e {
                    ClientExternalEvent::InitializeClient { transaction_id, metadata } => {
                        let response = self.state.external_initialize(transaction_id, metadata);
                        self.send(response);
                    }
                    ClientExternalEvent::Join { transaction_id, session_id } => {
                        let response = self.state.external_join(transaction_id, &session_id);
                        self.send(response);
                    }
                    ClientExternalEvent::MessageFromClient { transaction_id, data } => {
                        let response = self.state.external_message(transaction_id, data);
                        self.send(response);
                    }

                    _ => {
                        self.logger.warn(format!("Dispatch failed to process unknown message: {:?}", e));
                    }
                }
            }
            ClientEvent::Internal(e) => {
                match e {
                    ClientInternalEvent::ClientJoinResponse { transaction_id, success, error } => {
                        let response = self.state.internal_join_response(transaction_id, success, error);
                        self.send(response);
                    }
                    ClientInternalEvent::MessageFromClientResponse { transaction_id, success, error } => {
                        let response = self.state.internal_message_response(transaction_id, success, error);
                        self.send(response);
                    }
                    ClientInternalEvent::MessageFromMaster { data } => {
                        let response = self.state.internal_message_from_master(data);
                        self.send(response);
                    }
                    ClientInternalEvent::MasterDisconnected { reason } => {
                        let response = self.state.internal_master_disconnect(&reason);
                        self.send(response);
                        self.logger.warn(format!("Disconnected: Master disconnected: {}", reason));
                        return Err(());
                    }
                    _ => {
                        self.logger.warn(format!("Dispatch failed to process unknown message: {:?}", e));
                    }
                }
            }
            ClientEvent::Control(e) => {
                match e {
                    ClientControlEvent::Halt => return Err(()),
                    ClientControlEvent::ClientDisconnected { reason } => {
                        let response = self.state.external_disconnect(&reason);
                        self.send(response);
                        self.logger.warn(format!("Disconnected: {}", reason));
                        return Err(());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn event_loop(&mut self, channel: &IsolateChannel<ClientEvent>) -> Result<(), ()> {
        loop {
            match channel.receiver.recv() {
                Ok(event) => { self.dispatch(event)?; }
                Err(_err) => {
                    self.logger.warn("Channel closed, halting isolate");
                    return Err(());
                }
            }
        }
    }

    /// Send some arbitrary event to the appropriate destination and log it
    fn send(&self, dispatch: ClientEventDispatch) {
        match dispatch {
            ClientEventDispatch::DispatchNone => {}
            DispatchExternal(ext) => self.send_external(ext),
            DispatchInternal(event) => self.send_internal(event),
        }
    }

    fn send_external(&self, event: ClientExternalEvent) {
        match self.external.as_ref() {
            Some(channel) => {
                let output = ClientEvent::External(event);
                self.logger.outgoing_event(&output);
                let _ = channel.sender.send(output);
            }
            _ => {}
        };
    }

    fn send_internal(&self, event: MasterInternalEvent) {
        match self.state.master_ref() {
            Some(channel) => {
                let output = MasterEvent::Internal(event);
                self.logger.outgoing_event(&output);
                match channel.sender.send(output) {
                    Ok(_) => {}
                    Err(e) => {
                        self.logger.warn(format!("Failed to send internal event: {}", e.description()));
                    }
                }
            }
            _ => {
                self.logger.warn("An outgoing request was discarded because the client is not connected to a master");
            }
        };
    }
}

impl Isolate<ClientEvent> for ClientIsolate {
    fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<ClientEvent>) -> Box<FnMut() + Send + 'static> {
        let mut instance = self.instance(identity, &channel);
        Box::new(move || {
            let _ = instance.event_loop(&channel);
        })
    }
}