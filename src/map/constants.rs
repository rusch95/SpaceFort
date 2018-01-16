use map::tiles::Tile;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, PartialEq, Eq)]
pub enum Mode {
    Block,
    Empty,
    UpStairs,
    DownStairs,
    UpDownStairs,
    UpRamp,
    DownRamp,
}

pub fn to_mode(x: u32) -> Option<Mode> {
    use self::Mode::*;
    match x {
        0 => Some(Block),
        1 => Some(Empty),
        2 => Some(UpStairs),
        3 => Some(DownStairs),
        4 => Some(UpDownStairs),
        5 => Some(UpRamp),
        6 => Some(DownRamp),
        _ => None,
    }
}

pub const AIR_TILE: Tile = Tile {material: AIR_MAT, mode: Mode::Empty, marked: false};
pub const AIR_MAT: u16 = 10;
