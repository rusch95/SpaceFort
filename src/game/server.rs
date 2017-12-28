use std::net::TcpStream;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use std::thread;

use map::tiles::Map;
use game::base::*;
use entities::creatures::CreatureMap;
use entities::actions::{Action, Tasks, add_dig_tasks, Goal};
use entities::entity::{Entity, Entities, EntSnaps, EntID};
use entities::entity::{do_actions, resolve_dead, schedule_actions};
use entities::pathfind::{path_to, path_next_to};
use net::base::{ClientMsg, PlayerJoin};
use net::server::NetComm;


const VALIDATION_PERIOD: i32 = 10;

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

    pub fn attack(&mut self, player_id: PlayerID, attacker: EntID, 
                  defender: EntID) {
        if let Some(player) = self.players.get_mut(&player_id) {
            player.ents_attack(&[attacker], defender, &mut self.g_state);
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

        // Fix or null any invalid goals
        if self.ticks % VALIDATION_PERIOD == 0 {
            self.validate_goals();
        }
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

    pub fn validate_goals(&mut self) {
        use self::NewGoalPoss::*;

        // Generate new goals
        let new_goals: Vec<NewGoalPoss> = self.entities
            .iter()
            .filter(|ent| ent.goal.is_some())
            .map(|ent| {
                match ent.goal {
                    Some(Goal::Attack(attack_type, ent_id, pos)) => {
                        if let Some(t_ent) = self.entities.iter()
                                                          .find(|t_ent| t_ent.id == ent_id) {
                            if t_ent.pos != pos || ent.actions.is_empty() || 
                                    !t_ent.alive {
                                let goal = Goal::Attack(attack_type, ent_id, t_ent.pos);
                                NewGoal(ent.id, goal)
                            } else {
                                NoChange
                            }
                        } else {
                            Delete(ent.id)
                        }
                    },
                    None => NoChange,
                }})
            .filter(|goal| goal != &NoChange)
            .collect();

        // Implement new goals
        for goal in new_goals.iter() {
            match *goal {
                NewGoal(ent_id, Goal::Attack(attack_type, id, pos)) => {
                    let ent = self.entities.iter_mut()
                                           .find(|ent| ent.id == ent_id)
                                           .unwrap();
                    ent.actions = path_next_to(&self.map, ent, 
                                               &self.creature_types, pos);
                    let (action, goal) = Action::attack(id, pos, ent.creature_id,
                                                        &self.creature_types);
                    ent.actions.push_back(action);
                    ent.goal = Some(goal);
                },
                Delete(ent_id) => {
                    let ent = self.entities.iter_mut()
                                           .find(|ent| ent.id == ent_id)
                                           .unwrap();
                    ent.goal = None;
                    ent.actions.clear();
                },
                _ => unimplemented!(),
            }
        }
    }

    #[allow(dead_code)]
    pub fn give_id(&mut self) -> EntID {
        self.cur_id += 1;
        self.cur_id
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum NewGoalPoss {
    NoChange,
    Delete(EntID),
    NewGoal(EntID, Goal),
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
                    let (_, goal) = Action::attack(target.id, target.pos, ent.creature_id,
                                                   &g_state.creature_types);
                    ent.goal = Some(goal);
                }
            }
        }
    }
}
