#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate spacefort;
extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate pathfinding;
extern crate piston;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate toml;

use spacefort::*;

// Std lib imports
use std::path::Path;
use std::time::{Duration, Instant};

// Local imports
use game::base::init_game;
use map::tiles::init_map;
use entities::entity::init_entities;

const FRAME_RATE_NS: u32 = 1666667;


fn main() {   
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // REFACTOR Maybe should move non-essential inits into init_game
    let map = init_map(root);
    let (entities, creature_types) = init_entities(root);
    let mut game = init_game(map, entities, creature_types);

    // Game loop
    let mut now = Instant::now();
    let mut last_update = now;
    loop {
        game.player_update();

        // Updates per second
        now = Instant::now();
        if now.duration_since(last_update) >= Duration::new(0, FRAME_RATE_NS) {
            last_update = now;
            game.world_update();
        }
    }
}
