use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use piston::event_loop::*;
use piston::input::*;

use entities::creatures::CreatureMap;
use entities::entity::{Entities, EntID, EntSnaps};
use entities::interact::{select_entities, select_bad_entities};
use game::base::*;
use io::base::*;
use io::constants::*;
use io::utils::*;
use io::tiles::{render, init_graphics};
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use net::base::{ServerMsg, PlayerJoin};
use net::client::NetComm;


pub struct Client {
    pub player_id: PlayerID,

    pub ch: CameraHandle,
    mouse_pos: WinPos,
    pub selector: Option<Selector>,
    pub selected_entities: Vec<EntID>,
    selector_start: Option<WinPos>, 
    sel_state: SelState,
    pub done: bool,
    
    // Graphics and IO
    events: Events,
    pub gl: GlGraphics,
    window: Window,
    comm: NetComm,

    // State to sync from GameState
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub map: Map,
    pub ticks: Ticks,
}

pub fn init_client(map: Map, entities: Entities, creature_types: CreatureMap,
                   comm: NetComm) -> Client {
    Client::new(0, map, entities, creature_types, comm)
}

impl Client {
    // Top level global state
    pub fn new(player_id: PlayerID, map: Map,  entities: Entities, 
               creature_types: CreatureMap, comm: NetComm) -> Client {
        let mut events = Events::new(EventSettings::new());
        events.set_ups(240);

        Client {
            player_id: player_id,
            ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
            selected_entities: Vec::new(),
            mouse_pos: (0.0, 0.0),
            selector: None,
            selector_start: None,
            sel_state: SelState::Ents,
            done: false,

            window: init_graphics(),
            events: events,
            gl: GlGraphics::new(OPEN_GL_VERSION),
            comm: comm,

            map: map,
            creature_types: creature_types,
            entities: entities,
            ticks: 0,
        }
    }

    pub fn start(&mut self) {
        info!("Client Started");

        self.comm.ent_move(-1, (1, 1, 1));

        loop {
            if let Some(e) = self.events.next(&mut self.window) {

                if let Some(button) = e.press_args() {
                    self.press_button(button);
                }

                if let Some(pos) = e.mouse_cursor_args() {
                    self.move_cursor(pos);
                }

                if let Some(button) = e.release_args() {
                    self.release_button(button);
                }

                if let Some(_) = e.update_args() {
                    // Network Updates
                    while let Some(msg) = self.comm.get_incoming_msgs() {
                        self.dispatch(msg);
                    }
                }

                if let Some(r) = e.render_args() {
                    self.render(&r);
                }
            }

            if self.done {
                break;
            }
        }
    }

    pub fn press_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            self.selector_start = Some(self.mouse_pos);
            self.selector = Some((self.mouse_pos, self.mouse_pos))
        }

        if let Button::Keyboard(key) = button {
            let func = match key {
                Key::Right  | Key::L => Client::right,
                Key::Left   | Key::H => Client::left, 
                Key::Down   | Key::J => Client::back,
                Key::Up     | Key::K => Client::forward, 
                Key::Period | Key::O => Client::up,
                Key::Comma  | Key::P => Client::down,
                Key::A      => Client::attack_mode,
                Key::D      => Client::digging_mode,
                Key::Y      => Client::move_to,
                _           => Client::null,
            };

            func(self);
        }
    }

    pub fn release_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            if let Some(selector) = self.selector {   
                let tiles_selector = win_to_tile_selector(selector, &self.ch);

                // Convert to RPCs
                match self.sel_state {
                    SelState::Ents => {
                        self.selected_entities = 
                            select_entities(
                                &self.entities, 
                                self.player_id,
                                tiles_selector);
                    },
                    SelState::Digging  => {
                        self.comm.mark_dig(tiles_selector);
                        self.sel_state = SelState::Ents;
                    },
                    SelState::Attack => {
                        self.add_attack_goal(tiles_selector);
                        self.sel_state = SelState::Ents;
                    }
                }

                self.selector_start = None;
                self.selector = None;
            }
        }
    }

    fn add_attack_goal(&mut self, tiles_selector: TilesSelector) {
        let mut targets = select_bad_entities(
                              &self.entities,
                              self.player_id,
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
            ServerMsg::SendMapChunk(chunk) => self.map.apply_chunk(chunk),
            ServerMsg::UpdateTile(tile, pos) => self.map.update_tile(tile, pos),
            ServerMsg::Boot() => {
                warn!("Booted");
                self.done = true;
            }
            _ => {info!("Received {:?}", msg)},
        };
    }

    fn join(&mut self, player_join: PlayerJoin) {
        self.player_id = player_join.player_id;
        info!("Joined as Player {}", self.player_id);

        self.map.resize(player_join.map_dim);

        self.comm.request_map(((0, 0, 0), (0, 0, 0)));
    }

    fn update_ents(&mut self, ent_snaps: EntSnaps) {
        for ent_snap in ent_snaps {
            for ent in self.entities.iter_mut() {
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

    pub fn digging_mode(&mut self) {
        self.sel_state = SelState::Digging;
    }

    pub fn attack_mode(&mut self) {
        self.sel_state = SelState::Attack;
    }

    pub fn move_cursor(&mut self, pos: [f64; 2]) {
        self.mouse_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.selector_start {
            self.selector = Some((selector_pos, self.mouse_pos));
        }
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

    pub fn render(&mut self, r: &RenderArgs) {
        render(self, r);
    }
}
