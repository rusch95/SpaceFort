// Std lib imports
use std::net::Ipv4Addr;
use std::path::Path;

// Crate imports
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;
use piston::event_loop::*;
use piston::input::*;

// Local imports
use entities::creatures::CreatureMap;
use entities::entity::*;
use entities::actions::{select_entities};
use game::base::*;
use io::base::*;
use io::constants::*;
use io::utils::*;
use io::textures::*;
use io::tiles::{render, init_graphics};
use map::tiles::*;
use net::base::{ServerMsg, PlayerJoin};
use net::client::*;


const CLICK_THRESH: f64 = 40.0;

pub struct Client {
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
    events: Events,
    pub gl: GlGraphics,
    window: Window,
    pub textures: Textures, 
    comm: NetComm,

    // State to sync from GameState
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub map: Map,
    pub ticks: Ticks,
}

pub fn init_client(root: &Path, server_ip: Ipv4Addr) -> Client {
    // The client starts with an unsized blank map that 
    // is then resized onced connected to a server and is 
    // then populated with chunks downloaded from the server
    let map = blank_map(root);

    // Other initializations
    let (entities, creature_types) = init_entities(root);
    let window = init_graphics();
    let comm = init_network(server_ip);

    // Must be done after window creation for OpenGL reasons
    let textures = load_textures(root);

    info!("Done initializing client");
    Client::new(map, entities, creature_types, comm, window, textures)
}

impl Client {
    // Top level global state
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap,
               comm: NetComm, window: Window, textures: Textures) -> Client {

        // Initializations
        let mut events = Events::new(EventSettings::new());
        events.set_ups(240);

        Client {
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

            window: window,
            textures: textures,
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
        info!("Starting client");

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

    pub fn move_cursor(&mut self, pos: [f64; 2]) {
        self.mouse_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.selector_start {
            self.selector = Some((selector_pos, self.mouse_pos));
        }
    }

    pub fn release_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {

            if let Some(selector) = self.selector {   
                // Check for click on same spot
                if sel_dist(selector) < CLICK_THRESH {
                    let (pos, _) = selector;
                    let tile_pos = win_pos_to_tile(pos, &self.ch);

                    if let Some(ent) = self.entities.iter()
                                                    .rev()
                                                    .find(|ent| ent.pos == tile_pos) {
                        self.selected_entities = vec![ent.id];
                    } else {
                        self.selected_entities = Vec::new();
                    }

                } else {
                    let tiles_selector = win_to_tile_selector(selector, &self.ch);

                    match self.sel_state {
                        SelState::Ents => {
                            self.selected_entities = 
                                select_entities(
                                    |ent| ent.team_id == self.team_id,
                                    &self.entities, 
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
                }

                self.selector_start = None;
                self.selector = None;
            }
        }
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

    pub fn render(&mut self, r: &RenderArgs) {
        render(self, r);
    }
}
