use game::base::*;
use entities::entity::{EntID, EntSnaps};
use map::tiles::{Tile, MapChunk};

pub const SERVER_PORT: u16 = 9999;
pub const CLIENT_PORT: u16 = 0;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientMsg {
    Heartbeat(),
    Ack(),
    AskJoin(),
    RequestMap((Pos, Pos)),
    RequestEnts(),
    MarkDig((Pos, Pos)),
    EntAttack(EntID, EntID),
    EntMove(EntID, Pos),
    Leave(),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ServerMsg {
    Heartbeat(),
    Ack(),
    ReplyJoin(PlayerJoin),
    SendMapChunk(MapChunk),
    UpdateTile(Tile, Pos),
    SendEnts(EntSnaps),
    Boot(),
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayerJoin {
    pub player_id: PlayerID,
    pub map_dim: Pos
}

impl PlayerJoin {

    pub fn new(player_id: PlayerID, map_dim: Pos) -> PlayerJoin {
        PlayerJoin { 
            player_id: player_id, 
            map_dim: map_dim,
        }
    }
}
