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
extern crate toml;

use std::path::Path;

use piston::event_loop::*;
use piston::input::*;

use io::tiles::{ init_graphics, init_game };
use map::tiles::init_map;
use entities::entity::init_entities;


fn main() {   
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let map = init_map(root);
    let ent_state = init_entities(root);
    let mut window = init_graphics();
    let mut game = init_game(map, ent_state);
    let mut events = Events::new(EventSettings::new());

    while let Some(e) = events.next(&mut window) {

        if let Some(button) = e.press_args() {
            game.press_button(button);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            game.input.move_cursor(pos);
        }

        if let Some(button) = e.release_args() {
            game.input.release_button(&mut game.state, button);
        }

        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            game.state.update(&u);
        }

        if game.done {
            break;
        }
    }
}
