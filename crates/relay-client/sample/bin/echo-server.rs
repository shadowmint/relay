use relay_client::MasterOptions;
use relay_client::MasterTyped;
use relay_client::{AuthOptions, BackendType};
use relay_client::{MasterEvent, RelayError};
use serde::{Deserialize, Serialize};

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
    let master = MasterTyped::<EchoEvent>::new(MasterOptions {
        master_id: "EchoMaster".to_string(),
        max_clients: 10,
        remote: "ws://127.0.0.1:9977".to_string(),
        backend: BackendType::WebSocket,
        auth: AuthOptions {
            key: "key1234567890".to_string(),
            secret: "secret1234567890".to_string(),
            session_expires_secs: 1800,
        },
    })
    .await?;

    println!("Service running...");
    let reader = master.channel();
    loop {
        match reader.recv() {
            Ok(event) => {
                respond_to(event, &master).await;
            }
            Err(e) => {
                println!("Runtime error: {:?}", e);
                break;
            }
        };
    }

    Ok(())
}

async fn respond_to(raw_event: MasterEvent<EchoEvent>, master: &MasterTyped<EchoEvent>) {
    match raw_event {
        MasterEvent::Internal { client_id, event } => match event {
            EchoEvent::Request { value } => {
                match master
                    .send(MasterEvent::Internal {
                        client_id,
                        event: EchoEvent::Echo {
                            value: format!("ECHO!!!! {}", value),
                        },
                    })
                    .await
                {
                    Ok(_) => {
                        println!("Sent: {:?}", value);
                    }
                    Err(e) => {
                        println!("Err: {:?}", e);
                    }
                }
            }
            _ => {}
        },
        MasterEvent::External(e) => {
            println!(": {:?}", e);
        }
    };
}
