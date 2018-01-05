extern crate spacefort;
#[macro_use]
extern crate log;
extern crate env_logger;

// Std lib imports
use std::net::Ipv4Addr;
use std::path::Path;

// Local imports
use spacefort::*;
#[cfg(feature = "default")]
use game::client::init_client;
#[cfg(feature = "term")]
use game::term_client::init_client;

/// Graphical 2d tile client binary for SpaceFort
fn main() {   
    env_logger::init().unwrap();
    info!("Initializing client");

    // Root points to the directory containing
    // static where assets are loaded from
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    debug!("Loading from root at {:?}", root);

    // TODO Change this to be a command line parameter
    let server_ip = Ipv4Addr::new(18, 248, 0, 121);

    init_client(root, server_ip).start();

    info!("Closing client");
}
