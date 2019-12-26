use relay_auth::AuthRequest;
use relay_core::events::client_event::ClientExternalEvent;
use relay_core::events::master_event::MasterExternalEvent;
use relay_core::model::client_metadata::ClientMetadata;
use relay_core::model::external_error::ExternalError;
use relay_core::model::master_metadata::MasterMetadata;
use relay_core::CLIENT;
use relay_core::MASTER;
use serde::Serialize;
use std::error::Error;
use std::fmt::Debug;

fn main() {
    // Sent by either client or master to auth a connection
    trace(
        "ALL",
        AuthRequest {
            expires: 12312312312,
            key: "public_key_1adfasdfasdf".to_string(),
            hash: Some(format!("12312321312312321")),
        },
    );

    // Sent by the client application to initialize a new session
    trace(
        CLIENT,
        ClientExternalEvent::InitializeClient {
            transaction_id: format!("123"),
            metadata: ClientMetadata {
                name: format!("Doug"),
            },
        },
    );

    // Sent by the application to notify about transaction state (ready, error, etc)
    trace(
        CLIENT,
        ClientExternalEvent::TransactionResult {
            transaction_id: format!("1234"),
            success: false,
            error: Some(ExternalError {
                error_code: 1,
                error_reason: format!("Some message"),
            }),
        },
    );

    // Sent by the application to notify about transaction state (ready, error, etc)
    trace(
        CLIENT,
        ClientExternalEvent::TransactionResult {
            transaction_id: format!("1234"),
            success: true,
            error: None,
        },
    );

    // Join a session by id
    trace(
        CLIENT,
        ClientExternalEvent::Join {
            transaction_id: format!("123"),
            session_id: format!("Hello-world-session"),
        },
    );

    // Send a message to the master, this is a fire and forget action
    trace(
        CLIENT,
        ClientExternalEvent::MessageFromClient {
            transaction_id: format!("123"),
            data: format!("hello"),
        },
    );

    // Recv a message from the master
    trace(
        CLIENT,
        ClientExternalEvent::MessageToClient {
            data: format!("hello"),
        },
    );

    // The internal master disconnected or booted this client
    // This is a notification event, not an action by the client.
    trace(
        CLIENT,
        ClientExternalEvent::MasterDisconnected {
            reason: format!("Master went away"),
        },
    );

    // Sent by the client application to initialize a new session
    trace(
        MASTER,
        MasterExternalEvent::InitializeMaster {
            transaction_id: format!("123"),
            metadata: MasterMetadata {
                master_id: format!("Some master"),
                max_clients: 4,
            },
        },
    );

    // Notify the master that a client joined
    trace(
        MASTER,
        MasterExternalEvent::ClientJoined {
            client_id: format!("123123-213123123"),
            name: format!("some person"),
        },
    );

    // Sent by the application to notify about transaction state (ready, error, etc)
    trace(
        MASTER,
        ClientExternalEvent::TransactionResult {
            transaction_id: format!("1234"),
            success: false,
            error: Some(ExternalError {
                error_code: 1,
                error_reason: format!("Some message"),
            }),
        },
    );

    // Sent by the application to notify about transaction state (ready, error, etc)
    trace(
        MASTER,
        ClientExternalEvent::TransactionResult {
            transaction_id: format!("1234"),
            success: true,
            error: None,
        },
    );

    // Sent by the client application to initialize a new session
    trace(
        MASTER,
        MasterExternalEvent::ClientDisconnected {
            client_id: format!("123123-213123123"),
            reason: format!("Client bad connection"),
        },
    );

    // Send a message to the external master
    trace(
        MASTER,
        MasterExternalEvent::MessageFromClient {
            client_id: format!("123123-213123123"),
            data: format!("Hello"),
        },
    );

    // Recv a message from the external master to send to a client
    trace(
        MASTER,
        MasterExternalEvent::MessageToClient {
            client_id: format!("123123-213123123"),
            transaction_id: format!("123123-2131231244"),
            data: format!("Hello"),
        },
    );
}

fn trace<T: Debug + Serialize>(context: &str, data: T) {
    let output = match serde_json::to_string(&data) {
        Ok(s) => s,
        Err(e) => format!("Serialization failed: {}", e),
    };
    println!("\n{}: {:?}\n{}", context, data, output);
}
