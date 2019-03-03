use relay::Server;
use relay::ServerConfig;
use std::env;
use getopts::Options;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config", "use a specific config file", "FILE");
    opts.optflag("h", "help", "print this help menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }

    let config = matches.opt_str("c").unwrap_or("relay.toml".to_string());
    match ServerConfig::try_from(&config) {
        Ok(server_config) => {
            let fullpath = fs::canonicalize(config.to_string()).map(|i| i.to_string_lossy().to_string()).unwrap_or(config);
            run_relay(fullpath, server_config)
        }
        Err(e) => {
            panic!(e.to_string())
        }
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn run_relay(config_path: String, config: ServerConfig) {
    println!("======> Relay!");
    println!("config: {}", config_path);
    println!("  bind: {}", config.bind);
    match Server::new().listen(config) {
        Ok(_) => {}
        Err(e) => { panic!("Failed to start relay: {}", e); }
    }
}