#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate pathfinding;
extern crate piston;
#[macro_use]
extern crate serde_derive;
extern crate toml;

mod entities;
mod gen;
mod io;
mod map;
mod net;
mod objects;

// Std lib imports
use std::path::Path;

// Crate imports
use piston::event_loop::*;
use piston::input::*;

// Local imports
use io::tiles::{ init_graphics, init_game };
use map::tiles::init_map;
use entities::entity::init_entities;


fn main() {   
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    // REFACTOR Maybe should move non-essential inits into init_game
    let map = init_map(root);
    let (entities, creature_types) = init_entities(root);
    let mut window = init_graphics();
    let mut game = init_game(map, entities, creature_types);
    let mut events = Events::new(EventSettings::new());

    // Game loop
    // REFACTOR Will need to abstract this for ascii and testing
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
