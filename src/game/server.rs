use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::{Arc, RwLock};
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::{Duration, Instant};

use map::tiles::Map;
use game::base::*;
use entities::creatures::CreatureMap;
use entities::entity::{Entity, Entities, EntID, EntIDs};
use entities::entity::{do_actions, resolve_dead, schedule_actions};
use entities::interact::{Action, Tasks};
use entities::pathfind::{path_to, path_next_to};
use net::base::{ClientMsg, PlayerJoin};
use net::server::ServerNetOut;

type ClientMsgReceiver = Receiver<(ClientMsg, SocketAddr)>;

const FRAME_RATE_NS: u32 = 1666667;

pub struct Server {
    pub g_state: GameState,
    pub players: Vec<ServerPlayer>,
    pub next_player_id: PlayerID,
    pub net_out: ServerNetOut,
    pub receiver: ClientMsgReceiver,
}

pub struct GameState {
    pub map: Map,
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub ticks: Ticks,
    #[allow(dead_code)]
    pub cur_id: EntID, // Global state for giving things ids
}

pub struct ServerPlayer {
    pub player_id: PlayerID,
    pub tasks: Tasks,
}

pub fn init_server(map: Map, entities: Entities, creature_types: CreatureMap, 
                   net_out: ServerNetOut, receiver: ClientMsgReceiver) -> Server {
    Server::new(map, entities, creature_types, net_out, receiver)
}

impl Server {
    // Top level global state
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap,
               net_out: ServerNetOut, receiver: ClientMsgReceiver) -> Server {
        Server {
            g_state: GameState::new(map, entities, creature_types),
            players: vec![ServerPlayer::new(1)],
            next_player_id: 0,
            net_out: net_out,
            receiver: receiver,
        }
    }

    pub fn start(&mut self) {
        info!("Server started");

        // Game loop
        let mut now = Instant::now();
        let mut last_update = now;
        loop {
            // Player Updates
            self.player_update();

            // World Updates
            now = Instant::now();
            if now.duration_since(last_update) >= Duration::new(0, FRAME_RATE_NS) {
                last_update = now;
                self.world_update();
            }

            // Network Updates
            self.ent_updates()

            let dur = Duration::new(0, 1000);
            match self.receiver.recv_timeout(dur) {
                Ok((msg, src)) => self.dispatch(msg, src),
                Err(err) => {},
            }
        }
    }

    fn ent_updates(&mut self) {
        ent_snaps = EntSnaps::new();
        for ent in self.g_state.entities.iter_mut() {
            ent_snaps(ent.snap());
        }
    }

    pub fn add_player(&mut self, conn: SocketAddr) {
        let player_id = self.next_player_id;
        self.next_player_id += 1;

        self.players.push(ServerPlayer::new(player_id));

        let player_join = PlayerJoin::new(player_id, self.g_state.map.size());
        
        self.net_out.reply_join(player_join, conn);
    }

    pub fn ent_move(&mut self, ent_id: EntID, pos: Pos) {
        let ent_ids = vec![ent_id];
        self.g_state.move_ents(&ent_ids, pos);
    }

    pub fn player_update(&mut self) {
        for player in &mut self.players {
            schedule_actions(&mut self.g_state.entities, &mut player.tasks,
                             &mut self.g_state.map, &self.g_state.creature_types)
        }
    }

    pub fn world_update(&mut self) {
        self.g_state.update();
    }

    pub fn dispatch(&mut self, msg: ClientMsg, src: SocketAddr) {

        info!("Msg: {:?}", msg);
        match msg {
            ClientMsg::AskJoin() => self.add_player(src),
            ClientMsg::EntMove(ent_id, pos) => self.ent_move(ent_id, pos),
            _ => {},
        }
    }
}

impl GameState {
    // Contains all state corresponding to a running game
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap) -> GameState {
        GameState {
            map: map,
            creature_types: creature_types,
            entities: entities,
            ticks: 0,
            cur_id: 0,
        }
    }

    pub fn update(&mut self) {
        self.ticks += 1;

        // Entity update and pathfinding
        do_actions(&mut self.entities, &mut self.map);
        resolve_dead(&mut self.entities);
    }

    fn move_ents(&mut self,  ent_ids: &EntIDs, dest_pos: Pos) {
        for ent in &mut self.entities {
            for ent_id in ent_ids {
                if ent.id == *ent_id {
                    ent.actions = path_to(&self.map, ent,
                                          &self.creature_types, dest_pos);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn give_id(&mut self) -> EntID {
        self.cur_id += 1;
        self.cur_id
    }
}

impl ServerPlayer {
    pub fn new(player_id: PlayerID) -> ServerPlayer {
        ServerPlayer {
            player_id: player_id,
            tasks: Vec::new(),
        }
    }

    pub fn ents_attack(&mut self, attackers: &EntIDs, target_id: EntID, g_state: &mut GameState) {
        let team_id = Some(self.player_id);
        let (mut team_ents, mut else_ents): (Vec<&mut Entity>, Vec<&mut Entity>) =
             g_state.entities.iter_mut()
                             .partition( |ent| ent.team_id == team_id);

        if let Some(mut target) = else_ents.iter_mut()
                                           .find(|ref ent| (*ent).id == target_id) {
            for ent_id in attackers {
                if let Some(mut ent) = team_ents.iter_mut()
                                                .find(|ref ent| (*ent).id == *ent_id) {
                    ent.actions = path_next_to(&g_state.map, &mut ent,
                                               &g_state.creature_types,
                                               target.pos);
                    ent.actions.push_back(Action::attack(target_id));
                }
            }
        }
    }
}
