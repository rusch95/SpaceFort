extern crate spacefort;
#[macro_use]
extern crate log;
extern crate env_logger;

// Std lib imports
use std::path::Path;
use std::net::Ipv4Addr;

// Local imports
use spacefort::*;
use game::client::init_client;
use map::tiles::blank_map;
use entities::entity::init_entities;
use net::client::init_network;


fn main() {   
    env_logger::init().unwrap();
    info!("Starting client");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let map = blank_map(root);
    let (entities, creature_types) = init_entities(root);
    let server_ip = Ipv4Addr::new(127, 0, 0, 1);
    let comm = init_network(server_ip);

    init_client(map, entities, creature_types, comm).start();

    info!("Closing client");
}
