use std::fs::File;
use std::io::{Read, Write, BufWriter, Error};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Tile {
    material: u16,
}

#[derive(Eq, PartialEq)]
pub struct Map {
    tiles: Vec<Tile>, 
    xsize: i32,
    ysize: i32,
    zsize: i32,
}

impl Map {

    pub fn print(&self) {
        for z in 0..self.zsize {
            for y in 0..self.ysize {
                for x in 0..self.xsize {
                    print!("{0}", self.get_tile(x, y, z).material % 10);
                }
                print!("\n");
            }
            print!("\n");
        }
    }

    pub fn get_tile(&self, x: i32, y: i32, z: i32) -> Tile {
        let index = (x + y * self.xsize + z * self.xsize * self.ysize) as usize;
        self.tiles[index]
    }

    pub fn save(&self, path: &str) -> Result<(), Error> {
        let f = try!(File::create(&path)); 
        let mut writer = BufWriter::new(&f);

        try!(write!(&mut writer, "{} {} {}\n", self.xsize, self.ysize, self.zsize));

        for z in 0..self.zsize {
            for y in 0..self.ysize {
                for x in 0..self.xsize {
                    try!(write!(&mut writer, "{} ", self.get_tile(x, y, z).material));
                }
                try!(write!(&mut writer, "\n"));
            }
            try!(write!(&mut writer, "\n"));
        }
        Ok(())
    }
}

pub fn test_map() -> Map {
    let def_tile = Tile {material: 60000};
    Map { tiles: vec![def_tile; 3125000], xsize: 250, ysize: 250, zsize: 50}
}

pub fn load_map(path: &str) -> Result<Map, Error> {

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

    Ok(Map {tiles: tiles, xsize: x, ysize: y, zsize: z})
}

