mod connection;
use connection::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let config = get_config(args).unwrap();

    check_config(&config).unwrap();
    
    send_file(config);
}
