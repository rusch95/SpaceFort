
extern crate ncurses;
mod io;
mod map;

use map::tiles::*;

fn main()
{
    let save_map = test_map();
    save_map.save(".unit_test.sfm").expect("Map save failure");
    let load_map = load_map(".unit_test.sfm").expect("Map load failure");
    assert!(save_map == load_map);
}
