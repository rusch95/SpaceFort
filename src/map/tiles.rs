use std::cmp::min;
use std::fs::File;
use std::path::Path;
use std::io::{Read, Write, BufWriter, Error};

use game::base::*;
use io::base::*;
use map::constants::*;
use map::material::*;


pub type Tiles = Vec<Tile>;

pub type PosUnit = i32;
const CHUNK_TILES_X: PosUnit = 8;
const CHUNK_TILES_Y: PosUnit = 8;
const CHUNK_TILES_Z: PosUnit = 1;

//TODO Clean up unwraps

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tile {
    //Single map unit
    pub material: MaterialID,
    pub mode: Mode,
    pub marked: bool,
}

#[derive(Clone)]
pub struct Map {
    //Holds the terrain info as a vector of tiles
    tiles: Tiles, 
    pub materials: Materials,
    xlen: PosUnit,
    ylen: PosUnit,
    zlen: PosUnit,
}

#[derive(Debug, Clone)]
pub struct MapSnapshot {
    //Represents a slice of the map 
    //to be be delivered to the rendering engine
    pub tiles: Tiles,
    pub xlen: PosUnit,
    pub ylen: PosUnit,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct MapChunk {
    pub tiles: Tiles,
    pub pos: Pos,
    pub xlen: PosUnit,
    pub ylen: PosUnit,
    pub zlen: PosUnit,
}

pub fn init_map(root: &Path) -> Map {
    info!("Initializing map");
    let test_path = root.join("static/inc/maps/smol_map_excel.sfm.csv");
    let path_str = test_path
                   .to_str()
                   .expect("Unicode decode error");

    // Load materials properties file
    let materials = init_materials(root);

    load_map(path_str, materials).expect("Could not load map")
}

impl Tile {
    fn new(material: MaterialID, mode: Mode) -> Tile {
        Tile { 
            material: material,
            mode: mode, 
            marked: false,
        }
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

    pub fn size(&self) -> Pos {
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

    pub fn get_chunk(&self, pos: Pos, size: Pos) -> MapChunk {
        let (x0, y0, z0) = pos;
        let (xlen, ylen, zlen) = size;


        let mut tiles = Tiles::new();
        for x in x0..(x0 + xlen) {
            for y in y0..(y0 + ylen) {
                for z in z0..(z0 + zlen) {
                    let index = self.coords_to_index((x, y, z));
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

    // TODO Add duplication factor
    pub fn to_chunks(&self) -> Vec<MapChunk> {
        let mut chunks = Vec::<MapChunk>::new();
        
        let x_chunks = Map::get_num_chunks(self.xlen, CHUNK_TILES_X);
        let y_chunks = Map::get_num_chunks(self.ylen, CHUNK_TILES_Y);
        let z_chunks = Map::get_num_chunks(self.zlen, CHUNK_TILES_Z);

        for dx in 0..x_chunks {
            for dy in 0..y_chunks {
                for dz in 0..z_chunks {
                    let x = dx * CHUNK_TILES_X;
                    let y = dy * CHUNK_TILES_Y;
                    let z = dz * CHUNK_TILES_Z;
                    let pos = (x, y, z);

                    let xlen = min(CHUNK_TILES_X, self.xlen - dx * CHUNK_TILES_X);
                    let ylen = min(CHUNK_TILES_Y, self.ylen - dy * CHUNK_TILES_Y);
                    let zlen = min(CHUNK_TILES_Z, self.zlen - dz * CHUNK_TILES_Z);
                    let size = (xlen, ylen, zlen);

                    chunks.push(self.get_chunk(pos, size))
                }
            }
        }

        chunks
    }

    fn get_num_chunks(map_len: PosUnit, chunk_len: PosUnit) -> PosUnit {
        if map_len % chunk_len == 0 {
            map_len / chunk_len
        } else {
            map_len / chunk_len + 1
        }
    }

    pub fn apply_chunk(&mut self, chunk: &MapChunk) {
        let (x0, y0, z0) = chunk.pos;

        let mut chunk_i = 0;
        for x in 0..chunk.xlen {
            for y in 0..chunk.ylen {
                for z in 0..chunk.zlen {
                    let mx = x + x0;
                    let my = (y + y0) * self.xlen;
                    let mz = (z + z0) * self.xlen * self.ylen;
                    let map_i = (mx + my + mz) as usize;
                    self.tiles[map_i] = chunk.tiles[chunk_i];
                    chunk_i += 1;
                }
            }
        }
    }

    /// Tile accesor method
    pub fn get_tile(&self, pos: Pos) -> Option<Tile> {
        if self.in_bounds(pos) { 
            let index = self.coords_to_index(pos);
            Some(self.tiles[index])
        } else {
            None
        }
    }

    /// Perform some mutable operation to a tile
    fn apply_tile_func<F>(&mut self, pos: Pos, func: F)
        where F: Fn(&mut Tile) {

        if self.in_bounds(pos) {
            let index = self.coords_to_index(pos);
            func(&mut self.tiles[index]);
        }
    }

    fn in_bounds(&self, pos: Pos) -> bool {
        let (x, y, z) = pos;
        !(0 > x || 0 > y || 0 > z || x >= self.xlen || y >= self.ylen || z >= self.zlen)
    }

    fn coords_to_index(&self, pos: Pos) -> usize {
        let (x, y, z) = pos;
        (x + y * self.xlen + z * self.xlen * self.ylen) as usize
    }

    pub fn update_tile(&mut self, new_tile: Tile, pos: Pos) {
        self.apply_tile_func(pos, |tile| {
            tile.material = new_tile.material;
            tile.marked = false;
        });
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
        if let Some(tile) = self.get_tile(pos) {
            match tile.mode {
                Mode::Block => false,
                _ => true,
            }
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
                let index = self.coords_to_index((x, y, 0));
                print!("{0}", self.tiles[index].material % 10);
            }
            println!();
        }
        println!();
    }

    fn coords_to_index(&self, pos: Pos) -> usize {
        let (x, y, _) = pos;
        (x + y * self.xlen) as usize
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

pub fn load_map(path: &str, materials: Materials) -> Result<Map, Error> {
    // Load map from file. Currently unversioned so take heed.
    // Map validation is not performed.
    let mut f = try!(File::open(&path));
    let mut contents = String::new(); 
    try!(f.read_to_string(&mut contents));

    let mut tiles = Vec::new();
    let (mut xlen, mut ylen, mut zlen) = (0i32, 0i32, 0i32); 
    for (i, line) in contents.lines().enumerate() {
        if i == 0 {
            let mut split_line = line.split(",");
            let version: i32 = split_line.next().unwrap().parse().unwrap();
            assert!(version >= 1);
            xlen = split_line.next().unwrap().parse().unwrap();
            ylen = split_line.next().unwrap().parse().unwrap();
            zlen = split_line.next().unwrap().parse().unwrap();
        } else {
            for word in line.split(",") {
                let mut word_parts = word.split(";");
                if let Some(material_str) = word_parts.next() {
                    if material_str.len() > 0 {
                        let material: u16 = material_str.parse().unwrap();
                        let mode: Mode = match word_parts.next() { 
                            Some(mode_str) => to_mode(mode_str.parse().unwrap()).unwrap(),
                            None => Mode::Block,
                        };
                        tiles.push(Tile::new(material, mode));
                    }
                }
            }
        }
    }


    Ok(Map {tiles: tiles, materials: materials, xlen: xlen, ylen: ylen, zlen: zlen})
}
