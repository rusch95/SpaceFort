#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bincode;
extern crate spacefort;

// Std lib imports
use std::net::Ipv4Addr;
use std::path::Path;

// Local imports
use spacefort::*;
use game::server::init_server;


fn main() {   
    env_logger::init().unwrap();
    info!("Initializing server");

    // Root points to the directory containing
    // static where assets are loaded from
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let server_ip = Ipv4Addr::new(0, 0, 0, 0);
    init_server(root, server_ip).start();

    info!("Closing server");
}
