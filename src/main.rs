mod error;
mod proxy;

use std::env;

use rsip::HostWithPort;

use crate::proxy::Proxy;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.as_slice() {
        [_, ip, port] => match HostWithPort::try_from(format!("{}:{}", ip, port)) {
            Ok(host_with_port) => match Proxy::new(host_with_port) {
                Ok(mut proxy) => proxy.run(),
                Err(e) => eprintln!("Failed to create proxy: {}", e),
            },
            Err(e) => {
                eprintln!("Invalid args: {}", e);
            }
        },
        _ => eprintln!("Usage: <program> <host> <port>"),
    }
}
