extern crate spacefort;
#[macro_use]
extern crate log;
extern crate env_logger;

// Std lib imports
use std::path::Path;
use std::net::Ipv4Addr;

// Local imports
use spacefort::*;
#[cfg(feature = "default")]
use game::client::init_client;
#[cfg(feature = "term")]
use game::term_client::init_client;
use map::tiles::blank_map;
use entities::entity::init_entities;
use net::client::init_network;

/// Graphical 2d tile client binary for SpaceFort
fn main() {   
    env_logger::init().unwrap();
    info!("Initializing client");

    // Root points to the directory containing
    // static where assets are loaded from
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    debug!("Loading from root at {:?}", root);

    // The client starts with an unsized blank map that 
    // is then resized onced connected to a server and is 
    // then populated with chunks downloaded from the server
    let map = blank_map(root);

    let (entities, creature_types) = init_entities(root);
    // TODO Change this to be a command line parameter
    let server_ip = Ipv4Addr::new(18, 248, 0, 121);
    let comm = init_network(server_ip);

    init_client(map, entities, creature_types, comm).start();

    info!("Closing client");
}
