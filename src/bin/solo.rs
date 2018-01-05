#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bincode;
extern crate spacefort;

// Std lib imports
use std::net::Ipv4Addr;
use std::path::Path;
use std::thread;

// Local imports
use spacefort::*;
use game::server::init_server;
#[cfg(feature = "default")]
use game::client::init_client;
#[cfg(feature = "term")]
use game::term_client::init_client;


fn main() {   
    env_logger::init().unwrap();
    info!("Starting solo");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let server_ip = Ipv4Addr::new(0, 0, 0, 0);
    thread::spawn(move|| {
        info!("Starting server");
        init_server(root, server_ip).start();
    });

    info!("Starting client");
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    init_client(root, localhost).start();

    info!("Closing solo");
}
