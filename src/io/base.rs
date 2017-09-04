use entities::entity::Pos;


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SelState {
    Ents,
    Digging,
}


pub struct CameraHandle {
    //Representation of the player's camera
    pub xlen: i32,
    pub ylen: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

pub type WinPos = (f64, f64);
pub type Selector = (WinPos, WinPos);
pub type TilesSelector = (Pos, Pos);
pub type Id = i64;
