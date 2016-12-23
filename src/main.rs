extern crate ncurses;

use ncurses::*;

struct Tile {
    material: Material,
    temp: f32,
}

struct Sceen {
    tiles: &[Tile],
    x_dim: i16,
    y_dim: i16,
}

enum Material {
    Wood,
    Metal,
    Air,
}

fn print_screen(slice: &[Tile]) {
    print!
}

fn

fn main()
{
    init_scr();

    let tile = Tile { material: Material::Wood, temp: 70.0 };
    print_tile(tile);
}


