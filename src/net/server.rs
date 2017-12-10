use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr, UdpSocket};
use std::collections::HashMap;
use std::io;

use bincode::{serialize, deserialize, Infinite};

use game::base::*;
use entities::entity::{EntSnaps};
use map::tiles::{Tile, MapChunk};
use net::base::*;

pub fn init_network() -> (ServerNetIn, ServerNetOut) {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let conn = SocketAddrV4::new(localhost , SERVER_PORT);
    let socket = UdpSocket::bind(conn).unwrap();

    let net_in = ServerNetIn::new(socket.try_clone().unwrap());
    let net_out = ServerNetOut::new(socket);

    (net_in, net_out)
}

pub struct ServerNetOut {
    socket: UdpSocket,
    player_conns: HashMap<PlayerID, SocketAddr>,
}

pub struct ServerNetIn {
    socket: UdpSocket,
}

impl ServerNetOut {

    pub fn new(socket: UdpSocket) -> ServerNetOut {
        ServerNetOut {
            socket: socket,
            player_conns: HashMap::new(),
        }
    }

    pub fn heartbeat(&self, player_id: PlayerID) {
        self.snd_msg(player_id, ServerMsg::Heartbeat());
    }

    pub fn ack(&self, player_id: PlayerID) {
        self.snd_msg(player_id, ServerMsg::Ack());
    }

    pub fn reply_join(&mut self, player_join: PlayerJoin, conn: SocketAddr) {
        let player_id = player_join.player_id;
        self.player_conns.insert(player_id, conn);

        self.snd_msg(player_id, ServerMsg::ReplyJoin(player_join));
    }

    pub fn send_map_chunk(&self, player_id: PlayerID, map_chunk: MapChunk) {
        self.snd_msg(player_id, ServerMsg::SendMapChunk(map_chunk));
    }

    pub fn update_tile(&self, player_id: PlayerID, tile_snap: Tile, pos: Pos) {
        self.snd_msg(player_id, ServerMsg::UpdateTile(tile_snap, pos));
    }

    pub fn send_ents(&self, player_id: PlayerID, ent_snaps: EntSnaps) {
        self.snd_msg(player_id, ServerMsg::SendEnts(ent_snaps));
    }

    pub fn boot(&self, player_id: PlayerID) {
        self.snd_msg(player_id, ServerMsg::Boot());
    }

    pub fn snd_msg(&self, player_id: PlayerID, msg: ServerMsg) {
        if let Some(dst) = self.player_conns.get(&player_id) {
            self.snd(msg, *dst);
        } else {
            error!("Could not find PlayerID {} in PlayerIPs", player_id);
        }
    }

    fn snd(&self, msg: ServerMsg, dest: SocketAddr) -> Result<(), io::Error> {
        let encoded: Vec<u8> = serialize(&msg, Infinite).unwrap();
        try!(self.socket.send_to(&encoded, dest));

        Ok(())
    }
}

impl ServerNetIn {

    pub fn new(socket: UdpSocket) -> ServerNetIn {
        ServerNetIn {
            socket: socket,
        }
    }

    pub fn rcv(&self) -> Result<(ClientMsg, SocketAddr), io::Error> {
        let mut buf = [0; 512];
        let (amt, src) = try!(self.socket.recv_from(&mut buf));
        let dec_msg: ClientMsg = deserialize(&buf[..amt]).unwrap();

        Ok((dec_msg, src))
    }
}
