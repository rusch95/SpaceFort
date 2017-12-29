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
use game::client::init_client;
use map::tiles::init_map;
use map::tiles::blank_map;
use entities::entity::init_entities;
use net::server::init_network as init_s_network;
use net::client::init_network as init_c_network;


fn main() {   
    env_logger::init().unwrap();
    info!("Starting solo");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // REFACTOR Maybe should move non-essential inits into init_game
    info!("Starting server binary");
    let map = init_map(root);
    let (s_entities, s_creature_types) = init_entities(root);
    let server_ip = Ipv4Addr::new(0, 0, 0, 0);
    let server_comm = init_s_network(server_ip);
    thread::spawn(move|| {
        init_server(map, s_entities, s_creature_types, server_comm).start();
    });

    info!("Starting client");
    let client_map = blank_map(root);
    let (c_entities, c_creature_types) = init_entities(root);
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let client_comm = init_c_network(localhost);
    init_client(client_map, c_entities, c_creature_types, client_comm).start();

    info!("Closing solo");
}
