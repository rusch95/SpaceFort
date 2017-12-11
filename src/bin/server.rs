#[macro_use]
extern crate log;
extern crate env_logger;
extern crate bincode;
extern crate spacefort;

// Std lib imports
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

// Local imports
use spacefort::*;
use game::server::init_server;
use map::tiles::init_map;
use entities::entity::init_entities;
use net::server::init_network;


fn main() {   
    env_logger::init().unwrap();
    info!("Starting server binary");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // REFACTOR Maybe should move non-essential inits into init_game
    let map = init_map(root);
    let (entities, creature_types) = init_entities(root);
    let (in_net, out_net) = init_network();

    let (sender, receiver) = channel();
    let mut game = init_server(map, entities, creature_types, out_net, receiver);

    thread::spawn(move|| {
        loop {
            match in_net.rcv() {
                Ok((msg, src)) => sender.send((msg, src)).unwrap(),
                Err(_) => {},
            }
        }
    });

    game.start();

    info!("Closing server");
}
