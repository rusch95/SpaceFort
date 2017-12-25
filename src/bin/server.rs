#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bincode;
extern crate spacefort;

// Std lib imports
use std::path::Path;

// Local imports
use spacefort::*;
use game::server::init_server;
use map::tiles::init_map;
use entities::entity::init_entities;
use net::server::init_network;


fn main() {   
    env_logger::init().unwrap();
    info!("Initializing server");

    // Root points to the directory containing
    // static where assets are loaded from
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // REFACTOR Maybe should move non-essential inits into init_game
    let map = init_map(root);
    let (entities, creature_types) = init_entities(root);
    let comm = init_network();

    init_server(map, entities, creature_types, comm).start();

    info!("Closing server");
}
