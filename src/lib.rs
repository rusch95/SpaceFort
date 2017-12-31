#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate bincode;
extern crate glutin_window;
extern crate graphics;
#[macro_use]
extern crate log;
#[cfg(feature = "term")]
extern crate ncurses;
extern crate opengl_graphics;
extern crate pathfinding;
extern crate piston;
extern crate piston_window;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod entities;
pub mod game;
pub mod gen;
pub mod io;
pub mod map;
pub mod net;
pub mod objects;
