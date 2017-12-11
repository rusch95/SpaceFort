extern crate spacefort;
#[macro_use]
extern crate log;
extern crate env_logger;

// Std lib imports
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;

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
    let (in_net, out_net) = init_network();

    let (sender, receiver) = channel();
    let mut client = init_client(map, entities, creature_types, out_net, receiver);

    thread::spawn(move|| {
        loop {
            match in_net.rcv() {
                Ok(msg) => sender.send(msg).unwrap(),
                Err(_) => {},
            }
        }
    });

    client.start();

    info!("Closing client");
}
