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

pub struct Game {
    pub g_state: GameState,
    pub players: Vec<Player>,
}

pub struct GameState {
    pub map: Map,
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub ticks: Ticks,
    #[allow(dead_code)]
    pub cur_id: EntID, // Global state for giving things ids
}

pub struct Player {
    pub player_id: PlayerID,
    
    window: Window,
    events: Events,
    pub gl: GlGraphics,

    pub ch: CameraHandle,
    pub selected_entities: Vec<EntID>,
    pub tasks: Tasks,

    mouse_pos: WinPos,
    pub selector: Option<Selector>,
    selector_start: Option<WinPos>, 
    sel_state: SelState,
}

pub fn init_game(map: Map, entities: Entities, creature_types: CreatureMap) -> Game {
    Game::new(map, entities, creature_types)
}

impl Game {
    // Top level global state
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap) -> Game {
        Game {
            g_state: GameState::new(map, entities, creature_types),
            players: vec![Player::new(1)],
        }
    }

    pub fn player_update(&mut self) {

        for player in &mut self.players {

            if let Some(e) = player.events.next(&mut player.window) {

                if let Some(button) = e.press_args() {
                    player.press_button(&mut self.g_state, button);
                }

                if let Some(pos) = e.mouse_cursor_args() {
                    player.move_cursor(pos);
                }

                if let Some(button) = e.release_args() {
                    player.release_button(button, &mut self.g_state);
                }

                if let Some(r) = e.render_args() {
                    player.render(&mut self.g_state, &r);
                }
            }
            
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

impl Player {
    pub fn new(player_id: PlayerID) -> Player {
        let mut events = Events::new(EventSettings::new());
        events.set_ups(240);

        Player {
            player_id: player_id,

            window: init_graphics(),
            events: events,
            gl: GlGraphics::new(OPEN_GL_VERSION),

            ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
            selected_entities: Vec::new(),
            tasks: Vec::new(),

            mouse_pos: (0.0, 0.0),
            selector: None,
            sel_state: SelState::Ents,
            selector_start: None,
        }
    }

    pub fn press_button(&mut self, g_state: &mut GameState, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            self.selector_start = Some(self.mouse_pos);
            self.selector = Some((self.mouse_pos, self.mouse_pos))
        }

        if let Button::Keyboard(key) = button {
            let func = match key {
                Key::Right  | Key::L => Player::right,
                Key::Left   | Key::H => Player::left, 
                Key::Down   | Key::J => Player::back,
                Key::Up     | Key::K => Player::forward, 
                Key::Period | Key::O => Player::up,
                Key::Comma  | Key::P => Player::down,
                Key::A      => Player::attack_with_selected,
                Key::D      => Player::set_digging,
                Key::Y      => Player::move_to,
                _           => Player::null,
            };

            func(self, g_state);
        }
    }

    pub fn release_button(&mut self, button: Button, g_state: &mut GameState) {
        if button == Button::Mouse(MouseButton::Left) {
            if let Some(selector) = self.selector {   
                let tiles_selector = win_to_tile_selector(selector, &self.ch);

                match self.sel_state {
                    SelState::Ents => {
                        self.selected_entities = 
                            select_entities(
                                &g_state.entities, 
                                self, 
                                tiles_selector);
                    },
                    SelState::Digging  => {
                        add_dig_tasks(
                            &mut self.tasks, 
                            &mut g_state.map, 
                            tiles_selector);
                        self.sel_state = SelState::Ents;
                    },
                    SelState::Attack => {
                        self.add_attack_goal(g_state, tiles_selector);
                        self.sel_state = SelState::Ents;
                    }
                }

                self.selector_start = None;
                self.selector = None;
            }
        }
    }

    pub fn add_attack_goal(&mut self, g_state: &mut GameState, tiles_selector: TilesSelector) {
        let mut targets = select_bad_entities(
                              &g_state.entities, 
                              &self,                                   
                              tiles_selector);
        
        let team_id = Some(self.player_id);
        let (mut team_ents, mut else_ents): (Vec<&mut Entity>, Vec<&mut Entity>) = 
            g_state.entities.iter_mut()
                                 .partition( |ent| ent.team_id == team_id);
                                                         
        if let Some(target_id) = targets.pop() {
            if let Some(mut target) = else_ents.iter_mut()
                                               .find(|ref ent| (*ent).id == target_id) {
                for ent_id in &self.selected_entities {
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

    fn move_selected_entities(&mut self, g_state: &mut GameState, mouse_pos: WinPos) {
        let dest_tile_pos = win_pos_to_tile(mouse_pos, &self.ch);

        for ent in &mut g_state.entities {
            for ent_id in &self.selected_entities {
                if ent.id == *ent_id {
                    ent.actions = path_to(&g_state.map, ent, 
                                          &g_state.creature_types, dest_tile_pos);
                }
            }
        }

        self.selected_entities.clear();
    }

    pub fn forward(&mut self, _g_state: &mut GameState) {
        self.ch.y -= 1;
    }

    pub fn back(&mut self, _g_state: &mut GameState) {
        self.ch.y += 1;
    }

    pub fn left(&mut self, _g_state: &mut GameState) {
        self.ch.x -= 1;
    }

    pub fn right(&mut self, _g_state: &mut GameState) {
        self.ch.x += 1;
    }

    pub fn up(&mut self, _g_state: &mut GameState) {
        self.ch.z -= 1;
    }

    pub fn down(&mut self, _g_state: &mut GameState) {
        self.ch.z += 1;
    }

    pub fn null(&mut self, _g_state: &mut GameState) {}

    pub fn set_digging(&mut self, _g_state: &mut GameState) {
        self.sel_state = SelState::Digging;
    }

    pub fn attack_with_selected(&mut self, _g_state: &mut GameState) {
        self.sel_state = SelState::Attack;
    }

    pub fn move_cursor(&mut self, pos: [f64; 2]) {
        self.mouse_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.selector_start {
            self.selector = Some((selector_pos, self.mouse_pos));
        }
    }

    pub fn move_to(&mut self, g_state: &mut GameState) {
        let mouse_pos = self.mouse_pos;
        self.move_selected_entities(g_state, mouse_pos);
    }

    pub fn get_snap(&mut self, map: &Map) -> MapSnapshot {
        handle_to_snapshot(&self.ch, map)
    }

    pub fn render(&mut self, g_state: &mut GameState, r: &RenderArgs) {
        render(self, g_state, r);
    }
}
