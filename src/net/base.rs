use std::sync::mpsc::{Receiver, Sender, SyncSender};
use std::net::{TcpStream};

use game::base::*;
use entities::entity::{EntID, EntSnaps};
use map::tiles::{Tile, MapChunk};

pub const SERVER_PORT: u16 = 9999;
pub const CLIENT_PORT: u16 = 0;
pub const MSG_BUF_SIZE: usize = 4096;

pub type SyncClientMsgSend = SyncSender<(ClientMsg, PlayerID)>;
pub type ClientMsgSend = Sender<(ClientMsg, PlayerID)>;
pub type ClientMsgRecv = Receiver<(ClientMsg, PlayerID)>;
pub type ServerMsgSend = Sender<(ServerMsg, PlayerID)>;
pub type ServerMsgRecv = Receiver<(ServerMsg, PlayerID)>;
pub type SendStream = Sender<(TcpStream, PlayerID)>;
pub type RecvStream = Receiver<(TcpStream, PlayerID)>;


#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub enum ClientMsg {
    Heartbeat(),
    Ack(),
    Join(PlayerID),
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

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy, Debug)]
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

pub fn usize_to_u8_array(x: usize) -> [u8;4] {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;
    return [b1, b2, b3, b4]
}

pub fn u8_array_to_usize(buf: &[u8], i: usize) -> usize {
    let b1 : usize = (buf[i] as usize) << 24;
    let b2 : usize = (buf[i+1] as usize) << 16;
    let b3 : usize = (buf[i+2] as usize) << 8;
    let b4 : usize = buf[i+3] as usize;
    return b1 + b2 + b3 + b4
}

