use entities::entity::EntID;
use map::tiles::PosUnit;

pub const FRAME_RATE_NS: u32 = 16_666_667;

pub type PlayerID = u16;
pub type TeamID = Option<PlayerID>;
pub type Pos = (PosUnit, PosUnit, PosUnit);
pub type Ticks = i32;

pub enum Change {
    TileChange(Pos),
    EntChange(EntID),
}

