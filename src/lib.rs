#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate pathfinding;
extern crate piston;
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
