use net::base::*;
use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
use std::io;

use bincode::{serialize, deserialize, Infinite};

use game::base::*;
use entities::entity::EntID;

pub struct ClientNetOut {
    socket: UdpSocket,
    server: SocketAddrV4,
}

pub struct ClientNetIn {
    socket: UdpSocket,
}

pub fn init_network() -> (ClientNetIn, ClientNetOut) {
    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let conn = SocketAddrV4::new(localhost , CLIENT_PORT);
    let server = SocketAddrV4::new(localhost , SERVER_PORT);
    let socket = UdpSocket::bind(conn).unwrap();

    let net_in = ClientNetIn::new(socket.try_clone().unwrap());
    let net_out = ClientNetOut::new(socket, server);

    (net_in, net_out)
}

impl ClientNetOut {

    pub fn new(socket: UdpSocket, server: SocketAddrV4) -> ClientNetOut {
        ClientNetOut { 
            socket: socket,
            server: server,
        }
    }

    pub fn heartbeat(&self) {
        self.snd_msg(ClientMsg::Heartbeat());
    }

    pub fn ack(&self) {
        self.snd_msg(ClientMsg::Ack());
    }

    pub fn ask_join(&self) {
        self.snd_msg(ClientMsg::AskJoin());
    }

    pub fn request_map(&self, selection: (Pos, Pos)) {
        self.snd_msg(ClientMsg::RequestMap(selection));
    }

    pub fn request_ents(&self) {
        self.snd_msg(ClientMsg::RequestEnts());
    }

    pub fn mark_dig(&self, selection: (Pos, Pos)) {
        self.snd_msg(ClientMsg::MarkDig(selection));
    }

    pub fn ent_attack(&self, attacker: EntID, defender: EntID) {
        self.snd_msg(ClientMsg::EntAttack(attacker, defender));
    }
    
    pub fn ent_move(&self, ent_id: EntID, pos: Pos) {
        self.snd_msg(ClientMsg::EntMove(ent_id, pos));
    }

    pub fn leave(&self) {
        self.snd_msg(ClientMsg::Leave());
    }

    pub fn snd_msg(&self, msg: ClientMsg) {
        self.snd(msg, self.server);
    }

    fn snd(&self, msg: ClientMsg, dest: SocketAddrV4) -> Result<(), io::Error> {
        let encoded: Vec<u8> = serialize(&msg, Infinite).unwrap();
        try!(self.socket.send_to(&encoded, dest));

        Ok(())
    }
}

impl ClientNetIn {

    pub fn new(socket: UdpSocket) -> ClientNetIn {
        ClientNetIn { 
            socket: socket,
        }
    }

    pub fn rcv(&self) -> Result<ServerMsg, io::Error> {
        let mut buf = [0; 512];
        let (amt, src) = try!(self.socket.recv_from(&mut buf));
        let dec_msg: ServerMsg = deserialize(&buf[..amt]).unwrap();

        Ok(dec_msg)
    }
}
