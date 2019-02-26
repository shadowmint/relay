use relay::Server;
use relay::ServerConfig;

const BIND: &'static str = "0.0.0.0:9977";

fn main() {
    println!("Running on: {}", BIND);
    Server::new().listen(ServerConfig {
        bind: BIND.to_string()
    });
}