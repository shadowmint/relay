use relay_client::ClientEvent;
use relay_client::ClientOptions;
use relay_client::ClientTyped;
use relay_client::RelayError;
use relay_client::{AuthOptions, BackendType};
use serde::{Deserialize, Serialize};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum EchoEvent {
    Request { value: String },
    Echo { value: String },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match run().await {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e) as Box<dyn std::error::Error>),
    }
}

async fn run() -> Result<(), RelayError> {
    let client = ClientTyped::<EchoEvent>::new(ClientOptions {
        client_id: "Client".to_string(),
        session_id: "EchoMaster".to_string(),
        remote: "ws://127.0.0.1:9977".to_string(),
        backend: BackendType::WebSocket,
        auth: AuthOptions {
            key: "key1234567890".to_string(),
            secret: "secret1234567890".to_string(),
            session_expires_secs: 1800,
        },
    })
    .await?;

    for i in 1..10 {
        let reader = client.channel();
        client
            .send(ClientEvent::Internal(EchoEvent::Request {
                value: format!("Test: {}", i),
            }))
            .await?;
        match reader.recv_timeout(Duration::from_millis(1000)) {
            Ok(r) => {
                println!("Response: {:?}", r);
            }
            Err(_) => {}
        };
    }

    println!("Waiting for events to resolve before halting");
    thread::sleep(Duration::from_millis(5000));
    Ok(())
}
