mod master_state;

use rust_isolate::Isolate;
use rust_isolate::IsolateIdentity;
use rust_isolate::IsolateChannel;
use crate::events::client_event::ClientEvent;
use crate::events::master_event::MasterEvent;
use relay_logging::RelayEventLogger;
use crate::events::master_event::MasterExternalEvent;
use crate::infrastructure::services::SessionManager;
use crate::events::master_event::MasterControlEvent;
use crate::isolates::master::master_state::MasterState;
use crate::isolates::master::MasterEventDispatch::DispatchExternal;
use crate::events::master_event::MasterInternalEvent;
use crate::events::client_event::ClientInternalEvent;
use crate::{MASTER, NO_IDENTITY};
use crate::isolates::master::MasterEventDispatch::DispatchToClient;
use std::error::Error;

#[derive(Debug)]
pub enum MasterEventDispatch {
    DispatchNone,
    DispatchExternal(MasterExternalEvent),
    DispatchToClient(IsolateIdentity, ClientInternalEvent),
}

pub struct MasterIsolate {
    logger: RelayEventLogger,
    external: Option<IsolateChannel<MasterEvent>>,
    state: MasterState,
}

impl MasterIsolate {
    pub fn new(manager: SessionManager) -> MasterIsolate {
        let logger = RelayEventLogger::new(NO_IDENTITY, MASTER);
        MasterIsolate {
            state: MasterState::new(IsolateIdentity::new(), manager, logger.clone()),
            logger,
            external: None,
        }
    }

    pub fn instance(&self, identity: IsolateIdentity, channel: &IsolateChannel<MasterEvent>) -> MasterIsolate {
        let logger = RelayEventLogger::new(&identity.to_string(), MASTER);
        MasterIsolate {
            state: self.state.instance(identity, logger.clone()),
            logger,
            external: Some(channel.clone().unwrap()),
        }
    }

    pub fn dispatch(&mut self, event: MasterEvent) -> Result<(), ()> {
        self.logger.incoming_event(&event);
        match event {
            MasterEvent::External(e) => {
                match e {
                    MasterExternalEvent::InitializeMaster { transaction_id, metadata } => {
                        let response = self.state.external_initialize(transaction_id, metadata);
                        self.send(response);
                    }
                    MasterExternalEvent::MessageToClient { client_id, transaction_id, data } => {
                        let response = self.state.external_message_to_client(client_id, transaction_id, data);
                        self.send(response);
                    }

                    _ => {
                        self.logger.warn(format!("Dispatch failed to process unknown message: {:?}", e));
                    }
                }
            }
            MasterEvent::Internal(e) => {
                match e {
                    MasterInternalEvent::ClientJoinRequest { transaction_id, client_id, identity } => {
                        let response = self.state.internal_client_join_request(&client_id, transaction_id, identity);
                        self.send_many(response);
                    }
                    MasterInternalEvent::MessageFromClient { client_id, transaction_id, data } => {
                        let response = self.state.internal_client_message(client_id, transaction_id, data);
                        self.send_many(response);
                    }
                    MasterInternalEvent::ClientDisconnected { identity, reason } => {
                        let response = self.state.internal_client_disconnected(identity, &reason);
                        self.send(response);
                    }
                }
            }
            MasterEvent::Control(e) => {
                match e {
                    MasterControlEvent::Halt => return Err(()),
                    MasterControlEvent::MasterDisconnected { reason } => {
                        let response = self.state.external_master_disconnected(&reason);
                        self.send_many(response);
                        return Err(()); // Halt
                    }
                }
            }
        }
        Ok(())
    }

    pub fn event_loop(&mut self, channel: &IsolateChannel<MasterEvent>) -> Result<(), ()> {
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

    /// Send some arbitrary set of events to the appropriate destination and log them
    fn send_many(&self, dispatch: Vec<MasterEventDispatch>) {
        dispatch.into_iter().for_each(|i| self.send(i));
    }

    /// Send some arbitrary event to the appropriate destination and log it
    fn send(&self, dispatch: MasterEventDispatch) {
        match dispatch {
            MasterEventDispatch::DispatchNone => {}
            DispatchExternal(ext) => self.send_external(ext),
            DispatchToClient(identity, event) => self.send_to_client(identity, event),
        }
    }

    fn send_external(&self, event: MasterExternalEvent) {
        match self.external.as_ref() {
            Some(channel) => {
                let output = MasterEvent::External(event);
                self.logger.outgoing_event(&output);
                let _ = channel.sender.send(output);
            }
            _ => {}
        };
    }

    fn send_to_client(&self, identity: IsolateIdentity, event: ClientInternalEvent) {
        let output = ClientEvent::Internal(event);
        match self.state.get_client(&identity) {
            Some(client_channel) => {
                self.logger.outgoing_event(&output);
                match client_channel.sender.send(output) {
                    Ok(_) => {}
                    Err(e) => {
                        self.logger.warn(format!("Failed to send event to {:?}: {}", identity, e.description()));
                    }
                }
            }
            None => {
                // Fallback; we don't own this client, but maybe we need to send a message to it
                // anyway, for example, for rejected client messages.
                match self.state.get_external_client(&identity) {
                    Some(client_channel) => {
                        self.logger.outgoing_event(&output);
                        match client_channel.sender.send(output) {
                            Ok(_) => {}
                            Err(e) => {
                                self.logger.warn(format!("Failed to send event to {:?}: {}", identity, e.description()));
                            }
                        }
                    }
                    None => {
                        self.logger.warn(format!("Unable to send event to unknown client {:?}: {:?}", identity, output));
                    }
                }
            }
        }
    }
}

impl Isolate<MasterEvent> for MasterIsolate {
    fn spawn(&self, identity: IsolateIdentity, channel: IsolateChannel<MasterEvent>) -> Box<FnMut() + Send + 'static> {
        let mut instance = self.instance(identity, &channel);
        Box::new(move || {
            let _ = instance.event_loop(&channel);
        })
    }
}