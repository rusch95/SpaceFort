use std::fs::File;
use std::path::Path;
use std::io::{Read, Write, BufWriter, Error};

use game::base::*;
use io::base::CameraHandle;
use map::constants::*;
use map::material::{init_materials, MaterialID, Material, Materials};


pub type Tiles = Vec<Tile>;

//TODO Clean up unwraps

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tile {
    //Single map unit
    pub material: MaterialID,
    pub marked: bool,
}

#[derive(Clone, PartialEq)]
pub struct Map {
    //Holds the terrain info as a vector of tiles
    tiles: Tiles, 
    pub materials: Materials,
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

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MapChunk {
    pub tiles: Tiles,
    pub pos: Pos,
    pub xlen: i32,
    pub ylen: i32,
    pub zlen: i32,
}

pub fn init_map(root: &Path) -> Map {
    info!("Initializing map");
    let test_path = root.join("static/inc/maps/arena.sfm");
    let path_str = test_path
                   .to_str()
                   .expect("Unicode decode error");

    // Load materials properties file
    let materials = init_materials(root);

    load_map(path_str, materials).expect("Could not load map")
}

impl Tile {
    fn new(material: MaterialID) -> Tile {
        Tile { material: material,
               marked: false }
    }
}

impl Map {
    #[allow(dead_code)]
    pub fn print(&self) {
        //Debug print method
        //[debug] func
        for z in 0..self.zlen {
            for y in 0..self.ylen {
                for x in 0..self.xlen {
                    match self.get_tile((x, y, z)) {
                        Some(tile) => print!("{0}", tile.material % 10),
                        None       => print!(" "),
                    }
                }
                println!();
            }
            println!();
        }
    }

    pub fn size(&self) -> (i32, i32, i32) {
        (self.xlen, self.ylen, self.zlen)
    }

    // Resize map as given with blank tiles
    pub fn resize(&mut self, pos: Pos) {
        let (x, y, z) = pos;
        self.tiles = vec![AIR_TILE; (x * y * z) as usize];
        self.xlen = x;
        self.ylen = y;
        self.zlen = z;
    }

    pub fn get_chunk(&mut self, pos: Pos, size: Pos) -> MapChunk {
        let (x0, y0, z0) = pos;
        let (xlen, ylen, zlen) = size;

        let mut tiles = Tiles::new();
        for x in x0..(x0 + xlen) {
            for y in y0..(y0 + ylen) {
                for z in z0..(z0 + zlen) {
                    let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
                    tiles.push(self.tiles[index]);
                }
            }
        }

        MapChunk {
            tiles: tiles,
            pos: pos,
            xlen: xlen,
            ylen: ylen,
            zlen: zlen,
        }
    }

    pub fn apply_chunk(&mut self, chunk: MapChunk) {
        let (x0, y0, z0) = chunk.pos;

        for z in 0..chunk.zlen {
            for y in 0..chunk.ylen {
                for x in 0..chunk.xlen {
                    let chunk_i = (x + y * chunk.xlen + z * chunk.xlen * chunk.ylen) as usize;
                    let map_i = ((x + x0) + 
                                 (y + y0) * self.xlen + 
                                 (z + z0) * self.xlen * self.ylen) as usize;
                    self.tiles[map_i] = chunk.tiles[chunk_i];
                }
            }
        }
    }

    pub fn get_tile(&self, pos: Pos) -> Option<Tile> {
        //Tile accesor method
        let (x, y, z) = pos;
        if 0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen {
            None
        } else {
            let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
            Some(self.tiles[index])
        }
    }

    fn apply_tile_func<F>(&mut self, pos: Pos, func: F)
        where F: Fn(&mut Tile) {
        // Perform some mutable operation to a tile

        let (x, y, z) = pos;
        if 0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen {
            ()
        } else {
            let index = (x + y * self.xlen + z * self.xlen * self.ylen) as usize;
            func(&mut self.tiles[index]);
        }
    }

    pub fn dig(&mut self, pos: Pos) {
        let alt = self.get_alt(pos);
        self.apply_tile_func(pos, |tile| tile.material = alt);
    }

    pub fn mark(&mut self, pos: Pos) {
        self.apply_tile_func(pos, |tile| tile.marked = true);
    }

    #[allow(dead_code)]
    pub fn unmark(&mut self, pos: Pos) {
        self.apply_tile_func(pos, |tile| tile.marked = false);
    }

    fn grab_material(&self, pos: Pos) -> Option<Material> {
        if let Some(tile) = self.get_tile(pos) {
            if let Some(material) = self.materials.get(&tile.material) {
                Some(material.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_alt(&self, pos: Pos) -> MaterialID {
        if let Some(material) = self.grab_material(pos) {
            material.alt
        } else {
            0
        }
    }

    pub fn diggable(&self, pos: Pos) -> bool {
        if let Some(tile) = self.get_tile(pos) {
            if let Some(material) = self.materials.get(&tile.material) {
                material.diggable && !tile.marked
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn passable(&self, pos: Pos) -> bool {
        if let Some(material) = self.grab_material(pos) {
            material.passable
        } else {
            false
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
                                self.get_tile((x, y, z)).expect("Malformed map").material));
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
            println!();
        }
        println!();
    }
}

pub fn handle_to_snapshot(handle: &CameraHandle, map: &Map) -> MapSnapshot {
    //Uses handle and map to generate 2D snapshot
    //Eventually 3D snapshots may be enabled
    //Base interface method between rendering engine and map
    let mut tiles = Vec::with_capacity((handle.xlen * handle.ylen) as usize);
    for y in handle.y..handle.y + handle.ylen {
        for x in handle.x..handle.x + handle.xlen {
            match map.get_tile((x, y, handle.z)) {
                //Get_tile returns valid tile
                Some(tile) => tiles.push(tile),
                //Otherwise display as air
                None       => tiles.push(AIR_TILE),
            }
        }
    }
    MapSnapshot {tiles: tiles, xlen: handle.xlen, ylen: handle.ylen}
}

pub fn blank_map(root: &Path) -> Map {
    // Load materials properties file
    let materials = init_materials(root);

    Map {
        tiles: Tiles::new(), 
        materials: materials, 
        xlen: 0,
        ylen: 0,
        zlen: 0,
    }
}

fn load_map(path: &str, materials: Materials) -> Result<Map, Error> {
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


    Ok(Map {tiles: tiles, materials: materials, xlen: x, ylen: y, zlen: z})
}
