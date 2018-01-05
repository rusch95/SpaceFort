use std::net::Ipv4Addr;
use std::path::Path;

use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;

use game::base::*;
use entities::creatures::CreatureMap;
use entities::actions::{Action, Tasks, add_dig_tasks};
use entities::entity::{Entity, Entities, EntSnaps, EntID};
use entities::entity::schedule_actions;
use map::tiles::Map;
use net::base::{ClientMsg, PlayerJoin};
use net::server::NetComm;
use map::tiles::init_map;
use entities::entity::init_entities;
use net::server::init_network;


pub struct Server {
    pub g_state: GameState,
    pub players: HashMap<PlayerID, ServerPlayer>,
    pub comm: NetComm,
}

pub struct ServerPlayer {
    pub player_id: PlayerID,
    pub team_id: TeamID,
    pub tasks: Tasks,
}

pub fn init_server(root: &Path, server_ip: Ipv4Addr) -> Server {
    let map = init_map(root);
    let (entities, creature_types) = init_entities(root);
    let comm = init_network(server_ip);

    Server::new(map, entities, creature_types, comm)
}

impl Server {
    // Top level global state
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap,
               comm: NetComm) -> Server {
        Server {
            g_state: GameState::new(map, entities, creature_types),
            players: HashMap::new(),
            comm: comm,
        }
    }

    pub fn start(&mut self) {
        info!("Server started");

        // Game loop
        let mut now = Instant::now();
        loop {
            // Frame Rate Handler
            let time_elapsed = now.elapsed();
            let frame_dur = Duration::new(0, FRAME_RATE_NS);
            if time_elapsed < frame_dur {
                thread::sleep(frame_dur - time_elapsed);
            }
            now = Instant::now();

            self.update();
        }
    }

    pub fn update(&mut self) {
        // Network Updates
        while let Some((stream, player_id)) = self.comm.check_incoming_streams() {
            self.add_player(player_id, stream);
        }

        while let Some((msg, player_id)) = self.comm.check_incoming_msgs() {
            self.dispatch(msg, player_id);
        }

        // Player Updates
        self.player_update();

        // World Updates
        let mut changes = self.world_update();

        for change in changes.drain(..) {
            match change {
                Change::TileChange(pos) => self.tile_update(pos),
                Change::EntChange(_) => {},
            };
        }

        let player_ids: Vec<PlayerID> = self.players.keys().cloned().collect();
        for player_id in player_ids {
            self.ent_updates(player_id);
        }
    }

    fn ent_updates(&mut self, player_id: PlayerID) {
        let ent_snaps: EntSnaps = self.g_state.entities.iter_mut()
                                              .map(|ent| ent.snap())
                                              .collect();
        
        self.comm.send_ents(player_id, ent_snaps);
    }

    fn tile_update(&mut self, pos: Pos) {
        let tile_snap = self.g_state.map.get_tile(pos).unwrap();
        let player_ids: Vec<PlayerID> = self.players.keys().cloned().collect();

        for player_id in player_ids {
            self.comm.update_tile(player_id, tile_snap, pos);
        }
    }

    pub fn add_player(&mut self, player_id: PlayerID, stream: TcpStream) {
        self.players.insert(player_id, ServerPlayer::new(player_id, Some(player_id)));

        let player_join = PlayerJoin::new(player_id, Some(player_id), self.g_state.map.size());

        info!("Adding Player {} at addr {:?}", player_id, stream);
        self.comm.setup_out_stream((stream, player_id));
        self.comm.reply_join(player_id, player_join);

        self.send_map(player_id);
    }

    pub fn send_map(&mut self, player_id: PlayerID) {
        let chunks = self.g_state.map.to_chunks(); 
        for chunk in &chunks {
            self.comm.send_map_chunk(player_id, chunk);
        }
    }

    pub fn ent_move(&mut self, ent_id: EntID, pos: Pos) {
        let ent_ids = vec![ent_id];
        self.g_state.move_ents(&ent_ids, pos);
    }

    pub fn player_update(&mut self) {
        for player in self.players.values_mut() {
            schedule_actions(&mut self.g_state.entities, &mut player.tasks,
                             &self.g_state.map, &self.g_state.creature_types,
                             Some(player.player_id))
        }
    }

    pub fn world_update(&mut self) -> Vec<Change> {
        self.g_state.update()
    }

    pub fn dig(&mut self, player_id: PlayerID, selection: (Pos, Pos)) {
        if let Some(player) = self.players.get_mut(&player_id) {
            add_dig_tasks(&mut player.tasks, &mut self.g_state.map, selection);
        }
    }

    pub fn attack(&mut self, player_id: PlayerID, attacker_id: EntID, target_id: EntID) {
        if let Some(player) = self.players.get(&player_id) {
            let (mut attackers, mut defenders): (Vec<&mut Entity>, Vec<&mut Entity>) =
                 self.g_state.entities.iter_mut()
                                      .partition( |ent| ent.team_id == player.team_id);

            if let Some(target) = defenders.iter()
                                           .find(|ent| ent.id == target_id) {
                if let Some(mut attacker) = attackers.iter_mut()
                                                     .find(|ent| ent.id == attacker_id) {
                    let (_, goal) = Action::attack(target.id, target.pos, attacker.creature_id,
                                                   &self.g_state.creature_types);
                    attacker.goal = Some(goal);
                }
            }
        }
    }

    pub fn dispatch(&mut self, msg: ClientMsg, player_id: PlayerID) {
        debug!("Msg: {:?}", msg);
        match msg {
            ClientMsg::RequestMap(_) => self.send_map(0),
            ClientMsg::RequestEnts() => {},
            ClientMsg::MarkDig(sel) => self.dig(player_id, sel),
            ClientMsg::EntAttack(attacker, target) => self.attack(player_id, attacker, target),
            ClientMsg::EntMove(ent_id, pos) => self.ent_move(ent_id, pos),
            ClientMsg::Leave() => {}, 
            _ => unimplemented!(),
        }
    }
}

impl ServerPlayer {
    pub fn new(player_id: PlayerID, team_id: TeamID) -> ServerPlayer {
        ServerPlayer {
            player_id: player_id,
            team_id: team_id,
            tasks: Vec::new(),
        }
    }
}
