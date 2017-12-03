use std;

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
    pub p_state: PlayerState,
    pub done: bool,
}

pub struct GameState {
    pub map: Map,
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub ticks: Ticks,
    #[allow(dead_code)]
    pub cur_id: EntID, // Global state for giving things ids
}

pub struct PlayerState {
    pub player_id: PlayerID,
    
    pub window: Window,
    pub events: Events,
    pub gl: GlGraphics,

    pub ch: CameraHandle,
    pub selected_entities: Vec<EntID>,
    pub tasks: Tasks,

    pub mouse_pos: WinPos,
    pub selector: Option<Selector>,
    pub selector_start: Option<WinPos>, 
    pub sel_state: SelState,
}

impl PlayerState {
    pub fn new(player_id: PlayerID) -> PlayerState {
        let mut events = Events::new(EventSettings::new());
        events.set_ups(240);

        PlayerState {
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
}

pub fn init_game(map: Map, entities: Entities, creature_types: CreatureMap) -> Game {
    Game::new(map, entities, creature_types)
}

impl Game {
    // Top level global state
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap) -> Game {
        Game {
            g_state: GameState::new(map, entities, creature_types),
            p_state: PlayerState::new(1),
            done: false,
        }
    }

    pub fn player_update(&mut self) {

        if let Some(e) = self.p_state.events.next(&mut self.p_state.window) {

            if let Some(button) = e.press_args() {
                self.press_button(button);
            }

            if let Some(pos) = e.mouse_cursor_args() {
                self.move_cursor(pos);
            }

            if let Some(button) = e.release_args() {
                self.release_button(button);
            }

            if let Some(r) = e.render_args() {
                self.render(&r);
            }
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        render(self, args);
    }

    pub fn forward(&mut self) {
        self.p_state.ch.y -= 1;
    }

    pub fn back(&mut self) {
        self.p_state.ch.y += 1;
    }

    pub fn left(&mut self) {
        self.p_state.ch.x -= 1;
    }

    pub fn right(&mut self) {
        self.p_state.ch.x += 1;
    }

    pub fn up(&mut self) {
        self.p_state.ch.z -= 1;
    }

    pub fn down(&mut self) {
        self.p_state.ch.z += 1;
    }

    pub fn quit(&mut self) {
        self.done = true;
    }

    pub fn set_digging(&mut self) {
        self.p_state.sel_state = SelState::Digging;
    }

    pub fn attack_with_selected(&mut self) {
        self.p_state.sel_state = SelState::Attack;
    }

    pub fn move_to(&mut self) {
        let mouse_pos = self.p_state.mouse_pos;
        self.move_selected_entities(mouse_pos);
    }

    pub fn null(&mut self) {}

    pub fn press_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            self.p_state.selector_start = Some(self.p_state.mouse_pos);
            self.p_state.selector = Some((self.p_state.mouse_pos, 
                                                self.p_state.mouse_pos))
        }

        if let Button::Keyboard(key) = button {
            let func = match key {
                Key::Right  | Key::L => Game::right,
                Key::Left   | Key::H => Game::left, 
                Key::Down   | Key::J => Game::back,
                Key::Up     | Key::K => Game::forward, 
                Key::Period | Key::O => Game::up,
                Key::Comma  | Key::P => Game::down,
                Key::A      => Game::attack_with_selected,
                Key::D      => Game::set_digging,
                Key::Q      => Game::quit,
                Key::Y      => Game::move_to,
                _           => Game::null,
            };

            func(self);
        }
    }

    pub fn release_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            if let Some(selector) = self.p_state.selector {   
                let tiles_selector = win_to_tile_selector(selector, &self.p_state.ch);

                match self.p_state.sel_state {
                    SelState::Ents => {
                        self.p_state.selected_entities = 
                            select_entities(
                                &self.g_state.entities, 
                                &self.p_state,                                   
                                tiles_selector);
                    },
                    SelState::Digging  => {
                        add_dig_tasks(
                            &mut self.p_state.tasks, 
                            &mut self.g_state.map, 
                            tiles_selector);
                        self.p_state.sel_state = SelState::Ents;
                    },
                    SelState::Attack => {
                        self.add_attack_goal(tiles_selector);
                        self.p_state.sel_state = SelState::Ents;
                    }
                }

                self.p_state.selector_start = None;
                self.p_state.selector = None;
            }
        }
    }

    fn move_selected_entities(&mut self, mouse_pos: WinPos) {
        let dest_tile_pos = win_pos_to_tile(mouse_pos, &self.p_state.ch);

        for ent in &mut self.g_state.entities {
            for ent_id in &self.p_state.selected_entities {
                if ent.id == *ent_id {
                    ent.actions = path_to(&self.g_state.map, ent, 
                                          &self.g_state.creature_types, dest_tile_pos);
                }
            }
        }

        self.p_state.selected_entities.clear();
    }

    pub fn add_attack_goal(&mut self, tiles_selector: TilesSelector) {
        let mut targets = select_bad_entities(
                              &self.g_state.entities, 
                              &self.p_state,                                   
                              tiles_selector);
        
        let team_id = Some(self.p_state.player_id);
        let (mut team_ents, mut else_ents): (Vec<&mut Entity>, Vec<&mut Entity>) = 
            self.g_state.entities.iter_mut()
                                 .partition( |ent| ent.team_id == team_id);
                                                         
        if let Some(target_id) = targets.pop() {
            if let Some(mut target) = else_ents.iter_mut()
                                               .find(|ref ent| (*ent).id == target_id) {
                for ent_id in &self.p_state.selected_entities {
                    if let Some(mut ent) = team_ents.iter_mut()
                                                    .find(|ref ent| (*ent).id == *ent_id) {
                        ent.actions = path_next_to(&self.g_state.map, &mut ent,
                                                   &self.g_state.creature_types,
                                                   target.pos);
                        ent.actions.push_back(Action::attack(target_id));
                    } 
                }
            }
        }
    }

    pub fn move_cursor(&mut self, pos: [f64; 2]) {
        self.p_state.mouse_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.p_state.selector_start {
            self.p_state.selector = Some((selector_pos, self.p_state.mouse_pos));
        }
    }

    pub fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.p_state.ch, &self.g_state.map)
    }

    pub fn world_update(&mut self) {
        self.g_state.update();
        schedule_actions(&mut self.g_state, &mut self.p_state);
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
