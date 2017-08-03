mod entities;
mod gen;
mod io;
mod map;
mod net;
mod objects;

#[cfg(feature = "tiles")]
extern crate piston;
#[cfg(feature = "tiles")]
extern crate graphics;
#[cfg(feature = "tiles")]
extern crate glutin_window;
#[cfg(feature = "tiles")]
extern crate opengl_graphics;
#[cfg(feature = "tiles")]
use io::tiles::init_graphics;

#[cfg(feature = "curses")]
extern crate ncurses;
#[cfg(feature = "curses")]
use io::term::init_graphics;

use std::path::Path;
use std::sync::{RwLock, Arc};
use std::thread;
use std::time;
use map::tiles::load_map;
use entities::entity::init_entities;


fn main() 
{
    let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("static/inc/maps/smol_map.sfm");
    let path_str = test_path
                   .to_str()
                   .expect("Unicode decode error");
    let map = Arc::new(RwLock::new(load_map(path_str).expect("Could not load map")));

    let map_ref_entities = map.clone();
    init_entities(map_ref_entities);

    let map_ref_graphics = map.clone();
    thread::spawn(move || {
        init_graphics(map_ref_graphics);
    });

    loop {
        std::thread::sleep(time::Duration::from_millis(1000));
    }
}
