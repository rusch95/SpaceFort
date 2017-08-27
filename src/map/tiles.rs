use std::fs::File;
use std::path::Path;
use std::io::{Read, Write, BufWriter, Error};

use entities::entity::Pos;
use io::base::CameraHandle;
use map::constants::*;

type Tiles = Vec<Tile>;
type Material = u16;

//TODO Clean up unwraps

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tile {
    //Single map unit
    pub material: Material,
    pub marked: bool,
}

impl Tile {
    fn new(material: Material) -> Tile {
        Tile { material: material,
               marked: false }
    }
}


#[derive(Clone, Eq, PartialEq)]
pub struct Map {
    //Holds the terrain info as a vector of tiles
    tiles: Tiles, 
    xlen: i32,
    ylen: i32,
    zlen: i32,
}

#[derive(Debug, Clone)]
pub struct MapSnapshot {
    //Represents a slice of the map 
    //to be be delivered to the rendering engine
    pub tiles: Tiles,
    pub xlen: i32,
    pub ylen: i32,
}

impl Map {
    #[allow(dead_code)]
    pub fn print(&self) {
        //Debug print method
        //[debug] func
        for z in 0..self.zlen {
            for y in 0..self.ylen {
                for x in 0..self.xlen {
                    match self.get_tile(x, y, z) {
                        Some(tile) => print!("{0}", tile.material % 10),
                        None       => print!(" "),
                    }
                }
                print!("\n");
            }
            print!("\n");
        }
    }

    fn do_thing_with_tile(&mut self, pos: Pos) {
        let (x, y, z) = pos;
        if 0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen {
            ()
        } else {
            let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
            self.tiles[index].material = 0;
        }
    }

    pub fn get_tile(&self, x: i32, y: i32, z: i32) -> Option<Tile> {
        //Tile accesor method
        if 0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen {
            None
        } else {
            let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
            Some(self.tiles[index])
        }
    }

    pub fn dig(&mut self, pos: Pos) {
        let (x, y, z) = pos;
        if 0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen {
            ()
        } else {
            let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
            self.tiles[index].material = 0;
        }
    }

    pub fn mark(&mut self, pos: Pos) {
        let (x, y, z) = pos;
        if 0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen {
            ()
        } else {
            let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
            self.tiles[index].marked = true;
        }
    }

    #[allow(dead_code)]
    pub fn save(&self, path: &str) -> Result<(), Error> {
        //Saves map as file. Currently unversioned, so take heed.
        let f = try!(File::create(&path)); 
        let mut writer = BufWriter::new(&f);

        try!(write!(&mut writer, "{} {} {}\n", self.xlen, self.ylen, self.zlen));

        for z in 0..self.zlen {
            for y in 0..self.ylen {
                for x in 0..self.xlen {
                    try!(write!(&mut writer, "{} ", 
                                self.get_tile(x, y, z).expect("Malformed map").material));
                }
                try!(write!(&mut writer, "\n"));
            }
            try!(write!(&mut writer, "\n"));
        }
        Ok(())
    }
}

impl MapSnapshot {
    #[allow(dead_code)]
    pub fn print(&self) {
        //MapSnapshot debug
        //[debug] func
        for y in 0..self.ylen {
            for x in 0..self.xlen {
                let index = (x + y * self.xlen) as usize;
                print!("{0}", self.tiles[index].material % 10);
            }
            print!("\n");
        }
        print!("\n");
    }
}

pub fn handle_to_snapshot(handle: &CameraHandle, map: &Map) -> MapSnapshot {
    //Uses handle and map to generate 2D snapshot
    //Eventually 3D snapshots may be enabled
    //Base interface method between rendering engine and map
    let mut tiles = Vec::with_capacity((handle.xlen * handle.ylen) as usize);
    for y in handle.y..handle.y + handle.ylen {
        for x in handle.x..handle.x + handle.xlen {
            match map.get_tile(x, y, handle.z) {
                //Get_tile returns valid tile
                Some(tile) => tiles.push(tile),
                //Otherwise display as air
                None       => tiles.push(AIR_TILE),
            }
        }
    }
    MapSnapshot {tiles: tiles, xlen: handle.xlen, ylen: handle.ylen}
}

#[allow(dead_code)]
pub fn test_map() -> Map {
    //Generate test map of a single material
    //[debug] func
    let default_tile = Tile::new(0);
    Map {tiles: vec![default_tile; 400000], xlen: 200, ylen: 200, zlen: 5}
}

pub fn init_map(root: &Path) -> Map {
    let test_path = root.join("static/inc/maps/smol_map.sfm");
    let path_str = test_path
                   .to_str()
                   .expect("Unicode decode error");
    load_map(path_str).expect("Could not load map")
}

fn load_map(path: &str) -> Result<Map, Error> {
    //Load map from file. Currently unversioned so take heed.
    //Map validation is not performed.
    let mut f = try!(File::open(&path));
    let mut contents = String::new(); 
    try!(f.read_to_string(&mut contents));

    let mut tiles = Vec::new();
    let (mut x, mut y, mut z) = (0i32, 0i32, 0i32); 
    for (i, line) in contents.lines().enumerate() {
        if i == 0 {
            let mut split_line = line.split_whitespace();
            x = split_line.next().unwrap().parse().unwrap();
            y = split_line.next().unwrap().parse().unwrap();
            z = split_line.next().unwrap().parse().unwrap();
        } else {
            for word in line.split_whitespace() {
                let number: u16 = word.parse().unwrap();
                tiles.push(Tile::new(number));
            }
        }
    }

    Ok(Map {tiles: tiles, xlen: x, ylen: y, zlen: z})
}
