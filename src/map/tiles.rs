use std::fs::File;
use std::io::{Read, Write, BufWriter, Error};

use io::term::{TermHandle};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tile {
    //Single map unit
    pub material: u16,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Map {
    //Holds the terrain info as a vector of tiles
    tiles: Vec<Tile>, 
    xlen: i32,
    ylen: i32,
    zlen: i32,
}

#[derive(Debug, Clone)]
pub struct MapSnapshot {
    //Represents a slice of the map 
    //to be be delivered to the rendering engine
    pub tiles: Vec<Tile>,
    pub xlen: i32,
    pub ylen: i32,
    //entities: Vec<Entities>
}

impl Map {
    #[allow(dead_code)]
    pub fn print(&self) {
        //Debug print method
        //[debug] func
        for z in 0..self.zlen {
            for y in 0..self.ylen {
                for x in 0..self.xlen {
                    print!("{0}", self.get_tile(x, y, z).unwrap().material % 10);
                }
                print!("\n");
            }
            print!("\n");
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

    #[allow(dead_code)]
    pub fn save(&self, path: &str) -> Result<(), Error> {
        //Saves map as file. Currently unversioned, so take heed.
        let f = try!(File::create(&path)); 
        let mut writer = BufWriter::new(&f);

        try!(write!(&mut writer, "{} {} {}\n", self.xlen, self.ylen, self.zlen));

        for z in 0..self.zlen {
            for y in 0..self.ylen {
                for x in 0..self.xlen {
                    try!(write!(&mut writer, "{} ", self.get_tile(x, y, z).unwrap().material));
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

pub fn handle_to_snapshot(handle: &TermHandle, map: &Map) -> MapSnapshot {
    //Uses handle and map to generate 2D snapshot
    //Eventually 3D snapshots may be enabled
    //Base interface method between rendering engine and map
    let mut tiles = Vec::new();
    for y in handle.y..handle.y + handle.ylen {
        for x in handle.x..handle.x + handle.xlen {
            match map.get_tile(x, y, handle.z) {
                //Get_tile returns valid tile
                Some(tile) => tiles.push(tile),
                //Otherwise display as air
                None       => tiles.push(air_tile()),
            }
        }
    }
    MapSnapshot {tiles: tiles, xlen: handle.xlen, ylen: handle.ylen}
}

fn air_tile() -> Tile {
    //Generate empty tile for filling error space
    Tile {material: 60000}
}

#[allow(dead_code)]
pub fn test_map() -> Map {
    //Generate test map of a single material
    //[debug] func
    let def_tile = Tile {material: 0};
    Map {tiles: vec![def_tile; 400000], xlen: 200, ylen: 200, zlen: 5}
}

pub fn load_map(path: &str) -> Result<Map, Error> {
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
                tiles.push(Tile {material: number});
            }
        }
    }

    Ok(Map {tiles: tiles, xlen: x, ylen: y, zlen: z})
}

