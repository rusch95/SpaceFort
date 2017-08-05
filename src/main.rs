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

use std::path::Path;

use io::tiles::init_graphics;
use map::tiles::load_map;
use entities::entity::init_entities;


fn main() 
{
    let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("static/inc/maps/smol_map.sfm");
    let path_str = test_path
                   .to_str()
                   .expect("Unicode decode error");
    let map = load_map(path_str).expect("Could not load map");

    let entities = init_entities();

    init_graphics(map, entities);
}
