mod entities;
mod gen;
mod io;
mod map;
mod net;
mod objects;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate pathfinding;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate toml;

use std::path::Path;

use io::tiles::init_graphics;
use map::tiles::init_map;
use entities::entity::init_entities;


fn main() {   
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let map = init_map(root);
    let entities = init_entities();

    init_graphics(map, entities);
}
