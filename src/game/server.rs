use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;

use map::tiles::Map;
use game::base::*;
use entities::creatures::CreatureMap;
use entities::entity::{Entity, Entities, EntSnaps, EntID};
use entities::entity::{do_actions, resolve_dead, schedule_actions};
use entities::actions::{Action, Tasks, add_dig_tasks};
use entities::pathfind::{path_to, path_next_to};
use net::base::{ClientMsg, PlayerJoin};
use net::server::NetComm;


pub struct Server {
    pub g_state: GameState,
    pub players: HashMap<PlayerID, ServerPlayer>,
    pub comm: NetComm,
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
                   comm: NetComm) -> Server {
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
        self.players.insert(player_id, ServerPlayer::new(player_id));

        let player_join = PlayerJoin::new(player_id, self.g_state.map.size());

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

    pub fn attack(&mut self, player_id: PlayerID, attacker: EntID, defender: EntID) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.ents_attack(&[attacker], defender, &mut self.g_state);
        }
    }

    pub fn dispatch(&mut self, msg: ClientMsg, player_id: PlayerID) {
        info!("Msg: {:?}", msg);
        match msg {
            ClientMsg::RequestMap(_) => self.send_map(0),
            ClientMsg::RequestEnts() => {},
            ClientMsg::MarkDig(sel) => self.dig(player_id, sel),
            ClientMsg::EntAttack(attacker, target) => self.attack(player_id, attacker, target),
            ClientMsg::EntMove(ent_id, pos) => self.ent_move(ent_id, pos),
            ClientMsg::Leave() => {}, 
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

    pub fn update(&mut self) -> Vec<Change> {
        self.ticks += 1;

        // Entity update and pathfinding
        let changes = do_actions(&mut self.entities, &mut self.map);
        resolve_dead(&mut self.entities);

        changes
    }

    fn move_ents(&mut self,  ent_ids: &[EntID], dest_pos: Pos) {
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

    pub fn ents_attack(&mut self, attackers: &[EntID], target_id: EntID, 
                       g_state: &mut GameState) {
        let team_id = Some(self.player_id);
        let (mut team_ents, mut else_ents): (Vec<&mut Entity>, Vec<&mut Entity>) =
             g_state.entities.iter_mut()
                             .partition( |ent| ent.team_id == team_id);

        if let Some(target) = else_ents.iter_mut()
                                           .find(|ent| (*ent).id == target_id) {
            for ent_id in attackers {
                if let Some(mut ent) = team_ents.iter_mut()
                                                .find(|ent| (*ent).id == *ent_id) {
                    ent.actions = path_next_to(&g_state.map, ent,
                                               &g_state.creature_types,
                                               target.pos);
                    ent.actions.push_back(Action::attack(target_id));
                }
            }
        }
    }
}
