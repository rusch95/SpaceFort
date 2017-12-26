use game::base::Pos;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SelState {
    Ents,
    Digging,
    Attack,
}

pub struct CameraHandle {
    //Representation of the player's camera
    pub xlen: i32,
    pub ylen: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl CameraHandle {
    pub fn in_bounds(&self, pos: &Pos) -> bool {
        let (x, y, z) = *pos;
        z == self.z &&
        (self.x <= x) && (x <= self.x + self.xlen) &&
        (self.y <= y) && (y <= self.y + self.ylen)
    }
}

pub type WinPos = (f64, f64);
pub type Selector = (WinPos, WinPos);
pub type TilesSelector = (Pos, Pos);

