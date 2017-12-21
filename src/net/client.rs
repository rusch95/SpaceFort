use std::io;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpStream};
use std::thread;
use std::sync::mpsc::channel;

use bincode::{deserialize, serialize, Infinite};

use game::base::*;
use entities::entity::EntID;
use net::base::*;

pub struct ClientNetIn {
    pub stream: TcpStream,
    pub send_incoming: ServerMsgSend,
}

pub struct ClientNetOut {
    pub stream: TcpStream,
    pub recv_outgoing: ClientMsgRecv,
}

pub struct NetComm {
    pub send_outgoing: ClientMsgSend,
    pub recv_incoming: ServerMsgRecv,
}


pub fn init_network() -> NetComm {
    let server_ip = Ipv4Addr::new(18, 248, 0, 121);
    let server = SocketAddrV4::new(server_ip, SERVER_PORT);

    info!("Connecting to {}", server);
    let mut stream = TcpStream::connect(server).unwrap();
    stream.set_nodelay(true);

    let (send_outgoing, recv_outgoing) = channel();
    let (send_incoming, recv_incoming) = channel();

    let mut net_out = ClientNetOut::new(stream.try_clone().unwrap(), recv_outgoing);
    let mut net_in = ClientNetIn::new(stream, send_incoming);

    // Outgoing message handler
    thread::spawn(move|| {
        net_out.outgoing();
    });
            
    // Incoming message handler
    thread::spawn(move|| {
        net_in.incoming();
    });

    NetComm::new(send_outgoing, recv_incoming)
}

impl ClientNetOut {

    pub fn new(stream: TcpStream, recv_outgoing: ClientMsgRecv) -> ClientNetOut {
        ClientNetOut { 
            stream: stream,
            recv_outgoing: recv_outgoing,
        }
    }

    pub fn outgoing(&mut self) {
        loop {
            if let Ok((msg, player_id)) = self.recv_outgoing.try_recv() {
                self.snd(msg);
            };
        };
    }

    fn snd(&mut self, msg: ClientMsg) {

        let mut buf = [0u8; MSG_BUF_SIZE];
        let encoded: Vec<u8> = serialize(&msg, Infinite).unwrap();
        let enc_size_u8s = usize_to_u8_array(encoded.len());
        let buf_len = encoded.len() + 4;

        buf[..4].clone_from_slice(&enc_size_u8s);
        buf[4..buf_len].clone_from_slice(&encoded);
        let _amt = self.stream.write(&buf[..buf_len]);
    }
}

impl ClientNetIn {

    pub fn new(stream: TcpStream, send_incoming: ServerMsgSend) -> ClientNetIn {
        ClientNetIn { 
            stream: stream,
            send_incoming: send_incoming,
        }
    }

    pub fn incoming(&mut self) {
        loop {
            match self.rcv() {
                Ok(msg) => { self.send_incoming.send((msg, 0)); },
                Err(_) => {},            
            }
        }
    }

    pub fn rcv(&mut self) -> Result<ServerMsg, io::Error> {
        let mut n_buf = [0u8; 4];
        let mut buf = [0u8; MSG_BUF_SIZE];

        try!(self.stream.read_exact(&mut n_buf));
        let n = u8_array_to_usize(&n_buf[..], 0);
        let amt = try!(self.stream.read(&mut buf[..n]));

        let decoded: ServerMsg = deserialize(&buf[..amt]).unwrap();

        Ok(decoded)
    }
}

impl NetComm {

    pub fn new(send_outgoing: ClientMsgSend, recv_incoming: ServerMsgRecv) -> NetComm {
        NetComm { 
            send_outgoing: send_outgoing,
            recv_incoming: recv_incoming,
        }
    }

    pub fn get_incoming_msgs(&mut self) -> Option<ServerMsg> {
        match self.recv_incoming.try_recv() {
            Ok((msg, _)) => Some(msg),
            Err(_) => None,
        }
    }

    pub fn heartbeat(&self) {
        self.snd_msg(ClientMsg::Heartbeat());
    }

    pub fn ack(&self) {
        self.snd_msg(ClientMsg::Ack());
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
        self.send_outgoing.send((msg, 0));
    }

    pub fn rcv(&self) {

    }
}
