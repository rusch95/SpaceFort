use entities::entity::EntID;

pub const FRAME_RATE_NS: u32 = 16666667;

pub type PlayerID = u16;
pub type TeamID = Option<PlayerID>;
pub type Pos = (i32, i32, i32);
pub type Ticks = i32;

pub enum Change {
    TileChange(Pos),
    EntChange(EntID),
}

