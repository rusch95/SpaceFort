use ncurses::*;

use entities::creatures::CreatureMap;
use entities::entity::{Entities, EntID, EntSnaps};
use entities::actions::{select_entities};
use game::base::*;
use io::base::*;
use io::constants::*;
use io::utils::*;
use io::term::*;
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use net::base::{ServerMsg, PlayerJoin};
use net::client::NetComm;


pub struct TermClient {
    pub player_id: Option<PlayerID>,
    pub team_id: TeamID,

    pub ch: CameraHandle,
    mouse_pos: WinPos,
    pub selector: Option<Selector>,
    // Keep track of click down to detect if entity has been clicked
    pub selected_entities: Vec<EntID>,
    // The starting coordinate for the selection rectangle
    selector_start: Option<WinPos>, 
    // State for the selection state machine
    sel_state: SelState,
    // Whether the client is finished or not, such as if it has been booted by the server
    pub done: bool,
    
    // Graphics and IO
    comm: NetComm,

    // State to sync from GameState
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub map: Map,
    pub ticks: Ticks,
}

pub fn init_client(map: Map, entities: Entities, creature_types: CreatureMap,
                   comm: NetComm) -> TermClient {
    TermClient::new(map, entities, creature_types, comm)
}

impl TermClient {
    // Top level global state
pub fn new(map: Map,  entities: Entities, 
               creature_types: CreatureMap, comm: NetComm) -> TermClient {
        TermClient {
            // Start with none and initialize when connected
            player_id: None,
            team_id: None,

            ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
            mouse_pos: (0.0, 0.0),
            selector: None,
            selected_entities: Vec::new(),
            selector_start: None,
            sel_state: SelState::Ents,
            done: false,

            comm: comm,

            map: map,
            creature_types: creature_types,
            entities: entities,
            ticks: 0,
        }
    }

    pub fn start(&mut self) {
        info!("Starting client");
        init_term();

        loop {

            if self.done {
                end_term();      
                break;
            }
        }
    }

    pub fn get_input(&mut self) {
        //Get keyboard input, updating TermHandle, and changing map as necessary
        //TODO Enable key bindings
        //TODO Allow char instead of raw ascii 
		let ch = getch();
        let func = match ch {
          KEY_LEFT  => TermClient::left,
          KEY_RIGHT => TermClient::right,
          KEY_UP    => TermClient::forward,
          KEY_DOWN  => TermClient::back,
          60        => TermClient::down,
          62        => TermClient::up,
          81        => TermClient::exit,
          _         => TermClient::null,
        };

        func(self);

        //Debuging info
        mvprintw(50, 5, &format!("TermHandle x:{}, y:{}, z:{}", 
                                 self.ch.x, self.ch.y, self.ch.z));
    }

    fn add_attack_goal(&mut self, tiles_selector: TilesSelector) {
        let mut targets = select_entities(
                              |ent| ent.team_id != self.team_id,
                              &self.entities,
                              tiles_selector);

        if let Some(target_id) = targets.pop() {
            for attacker_id in &self.selected_entities {
                self.comm.ent_attack(*attacker_id, target_id);
            }
        }

        self.selected_entities.clear();
    }

    pub fn dispatch(&mut self, msg: ServerMsg) {
        match msg {
            ServerMsg::ReplyJoin(player_join) => self.join(player_join),
            ServerMsg::SendEnts(ent_snaps) => self.update_ents(ent_snaps),
            ServerMsg::SendMapChunk(chunk) => self.map.apply_chunk(&chunk),
            ServerMsg::UpdateTile(tile, pos) => self.map.update_tile(tile, pos),
            ServerMsg::Boot() => {
                warn!("Booted");
                self.done = true;
            }
            _ =>  unimplemented!(),
        };
    }

    fn join(&mut self, player_join: PlayerJoin) {
        self.player_id = Some(player_join.player_id);
        self.team_id = player_join.team_id;
        // Currently there is no mulitplayer
        info!("Joined as Player {}", player_join.player_id);

        self.map.resize(player_join.map_dim);

        self.comm.request_map(((0, 0, 0), (0, 0, 0)));
    }

    fn update_ents(&mut self, ent_snaps: EntSnaps) {
        for ent_snap in ent_snaps {
            for ent in &mut self.entities {
                if ent_snap.id == ent.id {
                    ent.pos = ent_snap.pos;
                    ent.health = ent_snap.health;
                    ent.alive = ent_snap.alive;
                }
            }
        }
    }

    pub fn forward(&mut self) {
        self.ch.y -= 1;
    }

    pub fn back(&mut self) {
        self.ch.y += 1;
    }

    pub fn left(&mut self) {
        self.ch.x -= 1;
    }

    pub fn right(&mut self) {
        self.ch.x += 1;
    }

    pub fn up(&mut self) {
        self.ch.z -= 1;
    }

    pub fn down(&mut self) {
        self.ch.z += 1;
    }

    pub fn null(&mut self) {}

    pub fn exit(&mut self) {
        self.done = true; 
    }

    pub fn digging_mode(&mut self) {
        self.sel_state = SelState::Digging;
    }

    pub fn attack_mode(&mut self) {
        self.sel_state = SelState::Attack;
    }

    pub fn move_to(&mut self) {
        let dst_pos = win_pos_to_tile(self.mouse_pos, &self.ch);

        for ent_id in &self.selected_entities {
            self.comm.ent_move(*ent_id, dst_pos);
        }

        self.selected_entities.clear();
    }

    pub fn get_snap(&self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map)
    }
}
