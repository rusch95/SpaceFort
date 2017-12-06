use piston::event_loop::*;
use piston::input::*;
use opengl_graphics::GlGraphics;
use glutin_window::GlutinWindow as Window;

use io::base::*;
use io::constants::*;
use io::utils::*;
use io::tiles::{render, init_graphics};
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use entities::creatures::CreatureMap;
use entities::entity::{Entity, Entities, Ticks, do_actions, resolve_dead, schedule_actions};
use entities::interact::{Action, Tasks, select_entities, select_bad_entities, add_dig_tasks};
use entities::pathfind::{path_to, path_next_to};


pub type PlayerID = u16;
pub type TeamID = Option<PlayerID>;
pub type EntID = i64;

pub struct ServerGame {
    pub g_state: GameState,
    pub players: Vec<ServerPlayer>,
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

pub fn init_game(map: Map, entities: Entities, creature_types: CreatureMap) -> ServerGame {
    ServerGame::new(map, entities, creature_types)
}

impl ServerGame {
    // Top level global state
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap) -> ServerGame {
        ServerGame {
            g_state: GameState::new(map, entities, creature_types),
            players: vec![Player::new(1)],
        }
    }

    pub fn player_update(&mut self) {
        for player in &mut self.players {
            schedule_actions(&mut self.g_state, player);
        }
    }

    pub fn world_update(&mut self) {
        self.g_state.update();
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
        do_actions(self);
        resolve_dead(self);
    }

    #[allow(dead_code)]
    pub fn give_id(&mut self) -> EntID {
        self.cur_id += 1;
        self.cur_id
    }
}

impl ServerPlayer {
    pub fn new(player_id: PlayerID) -> Player {
        ServerPlayer {
            player_id: player_id,
            tasks: Vec::new(),
        }
    }
}
