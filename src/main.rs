extern crate ncurses;
mod io;
mod map;

use io::term::init_term;
use map::tiles::load_map;

#[allow(unused_mut)]
fn main() 
{
    let test_path = "/home/rusch/Projects/SpaceFort/static/maps/test_map.sfm" ;
    let mut term_handle = init_term();
    let mut map = load_map(test_path).expect("Could not load map");

    loop {
        term_handle.update_display(&map);
        if !term_handle.get_input() {
            break;
        }
    }
}
