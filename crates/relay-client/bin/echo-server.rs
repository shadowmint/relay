use futures::Future;
use relay_client::MasterEvent;
use relay_client::MasterOptions;
use relay_client::{AuthOptions, BackendType};
use relay_client::{MasterTyped};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "object_type")]
pub enum EchoEvent {
    Request { value: String },
    Echo { value: String },
}

fn main() {
    tokio::run(
        MasterTyped::<EchoEvent>::new(MasterOptions {
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
        .then(|m| {
            println!("Callback?");
            match m {
                Ok(master) => {
                    println!("Service running...");
                    let reader = master.channel();
                    loop {
                        match reader.recv() {
                            Ok(event) => {
                                respond_to(event, &master);
                            }
                            Err(e) => {
                                println!("Runtime error: {:?}", e);
                                break;
                            }
                        };
                    }
                }
                Err(e) => {
                    println!("Startup error: {:?}", e);
                }
            }
            Ok(())
        }),
    );
}

fn respond_to(raw_event: MasterEvent<EchoEvent>, master: &MasterTyped<EchoEvent>) {
    match raw_event {
        MasterEvent::Internal { client_id, event } => match event {
            EchoEvent::Request { value } => {
                tokio::spawn(
                    master
                        .send(MasterEvent::Internal {
                            client_id,
                            event: EchoEvent::Echo {
                                value: format!("ECHO!!!! {}", value),
                            },
                        })
                        .then(|r| {
                            println!("Send: {:?}", &r);
                            Ok(())
                        }),
                );
            }
            _ => {}
        },
        MasterEvent::External(e) => {
            println!(": {:?}", e);
        }
    }
}
