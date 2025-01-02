pub mod peer;
pub mod server;

use crate::peer::Peer;
use crate::server::*;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    match args[1].as_str() {
        "server" => {
            let port = args[2].clone();
            let server = Server::new(port);
            match server {
                Ok(server) => server.run(),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "peer" => {
            let port = &args[2];
            let peer_address = &args[3];
            let server_address = &args[4];
            let peer = Peer::new(port, peer_address, server_address);
            match peer {
                Ok(mut peer) => peer.run(),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Usage: cargo run <port> <peer_port>");
            std::process::exit(1);
        }
    }
}
