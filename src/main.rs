#[cfg(feature = "curses")]
extern crate ncurses;

extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;

mod io;
mod map;

use std::path::Path;
use std::sync::{RwLock, Arc};
use std::thread;
use std::time;
use map::tiles::load_map;
use io::tiles::init_graphics;

#[allow(unused_mut)]
fn main() 
{
    let test_path = Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("static/inc/maps/test_map.sfm");
    let path_str = test_path
                   .to_str()
                   .expect("Unicode decode error");
    let map = Arc::new(RwLock::new(load_map(path_str).expect("Could not load map")));

    let map_ref = map.clone();
    thread::spawn(move || {
        init_graphics(map_ref);
    });

    loop {
        std::thread::sleep(time::Duration::from_millis(1000));
    }
}
