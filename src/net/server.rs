use std::collections::HashMap;
use std::io;
use std::io::{Read, Write};
use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::sync::mpsc::{channel, sync_channel};
use std::thread;
use std::time::Duration;

use bincode::{deserialize, serialize, Infinite};

use game::base::*;
use entities::entity::{EntSnaps};
use map::tiles::{Tile, MapChunk};
use net::base::*;


pub struct ServerNetOut {
    pub player_conns: HashMap<PlayerID, TcpStream>,
    pub recv_outgoing: ServerMsgRecv,
    pub recv_stream_from_game: RecvStream,
}

pub struct NetComm {
    pub send_outgoing: ServerMsgSend,
    pub recv_incoming: ClientMsgRecv,
    pub recv_stream_to_game: RecvStream,
    pub send_stream_from_game: SendStream,
}

pub fn init_network() -> NetComm {

    let localhost = Ipv4Addr::new(127, 0, 0, 1);
    let conn = SocketAddrV4::new(localhost , SERVER_PORT);
    let listener = TcpListener::bind(conn).unwrap();

    let (send_outgoing, recv_outgoing) = channel();
    let (send_incoming, recv_incoming) = sync_channel(1024);

    // New Stream -> NetIn -> Game -> NetOut
    let (send_stream_to_game, recv_stream_to_game) = channel();
    let (send_stream_from_game, recv_stream_from_game) = channel();

    // Listener thread
    thread::spawn(move|| {
        let mut next_player_id = 0;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    stream.set_nodelay(true);

                    let player_id = next_player_id;
                    next_player_id += 1;

                    // Send copy of stream to outgoing
                    let stream_copy = stream.try_clone().unwrap();
                    send_stream_to_game.send((stream_copy, player_id)).unwrap();

                    let send_in_clone = send_incoming.clone();
                    thread::spawn(move|| {
                        info!("New client");
                        // Tell the game that  a new player has joined
                        handle_client(stream, send_in_clone, player_id);
                    });

                }

                Err(_) =>  {},
            }
        }
    });

    let mut net = ServerNetOut::new(recv_outgoing, recv_stream_from_game);
    thread::spawn(move || { net.outgoing() });

    NetComm::new(send_outgoing, recv_incoming,
                 send_stream_from_game, recv_stream_to_game)
}

pub fn handle_client(mut stream: TcpStream, send_incoming: SyncClientMsgSend, 
                     player_id: PlayerID) {
    loop {
        match rcv(&mut stream) {
            Ok(msg) => { 
                send_incoming.send((msg, player_id)).unwrap();
            },
            Err(err) => { 
                warn!("Client stream err: {}", err);
                break ;
            },            
        }
    }
}

pub fn rcv(stream: &mut TcpStream) -> Result<ClientMsg, io::Error> {
    let mut n_buf = [0u8; 4];
    let mut buf = [0u8; MSG_BUF_SIZE];

    try!(stream.read_exact(&mut n_buf));
    let n = u8_array_to_usize(&n_buf[..], 0);
    let amt = try!(stream.read(&mut buf[..n]));

    let decoded: ClientMsg = deserialize(&buf[..amt]).unwrap();

    Ok(decoded)
}

impl ServerNetOut {

    pub fn new(recv_outgoing: ServerMsgRecv, 
               recv_stream_from_game: RecvStream) -> ServerNetOut {
        ServerNetOut {
            player_conns: HashMap::new(),
            recv_outgoing: recv_outgoing,
            recv_stream_from_game: recv_stream_from_game,
        }
    }

    pub fn outgoing(&mut self) {
        loop {
            if let Ok((msg, player_id)) = self.recv_outgoing.recv() {
                self.snd(msg, player_id);
            };
        };
    }

    fn snd(&mut self, msg: ServerMsg, player_id: PlayerID) {
        // Iterate over new streams until the stream for this joining can be added
        // TODO Think about whether this checking is neccessary
        if let ServerMsg::ReplyJoin(reply) = msg {
            while !self.player_conns.contains_key(&reply.player_id) {
                match self.recv_stream_from_game.recv() {
                    Ok((stream, player_id)) => { self.player_conns.insert(player_id, stream); },
                    Err(_) => error!("Stream was not available for adding."),
                }
            }
        }

        if let Some(mut conn) = self.player_conns.get(&player_id) {

            let mut buf = [0u8; MSG_BUF_SIZE];
            let encoded: Vec<u8> = serialize(&msg, Infinite).unwrap();
            let enc_size_u8s = usize_to_u8_array(encoded.len());
            let buf_len = encoded.len() + 4;

            buf[..4].clone_from_slice(&enc_size_u8s);
            buf[4..buf_len].clone_from_slice(&encoded);
            let _amt = conn.write(&buf[..buf_len]);
        }
    }
}

impl NetComm {

    pub fn new(send_outgoing: ServerMsgSend, recv_incoming: ClientMsgRecv,
               send_stream_from_game: SendStream, recv_stream_to_game: RecvStream)
               -> NetComm {
        NetComm {
            send_outgoing: send_outgoing,
            recv_incoming: recv_incoming,
            recv_stream_to_game: recv_stream_to_game,
            send_stream_from_game: send_stream_from_game,
        }
    }

    pub fn check_incoming_streams(&mut self) -> Option<(TcpStream, PlayerID)> {
        match self.recv_stream_to_game.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }

    pub fn setup_out_stream(&mut self, msg: (TcpStream, PlayerID)) {
        self.send_stream_from_game.send(msg).unwrap();
    }

    pub fn check_incoming_msgs(&mut self) -> Option<(ClientMsg, PlayerID)> {
        match self.recv_incoming.try_recv() {
            Ok(msg) => Some(msg),
            Err(_) => None,
        }
    }

    pub fn reply_join(&self, player_id: PlayerID, player_join: PlayerJoin) {
        self.snd_msg(player_id, ServerMsg::ReplyJoin(player_join));
    }

    pub fn heartbeat(&self, player_id: PlayerID) {
        self.snd_msg(player_id, ServerMsg::Heartbeat());
    }

    pub fn ack(&self, player_id: PlayerID) {
        self.snd_msg(player_id, ServerMsg::Ack());
    }

    pub fn send_map_chunk(&self, player_id: PlayerID, map_chunk: &MapChunk) {
        self.snd_msg(player_id, ServerMsg::SendMapChunk(map_chunk.clone()));
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
        self.send_outgoing.send((msg, player_id)).unwrap();
    }
}
