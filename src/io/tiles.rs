use std::mem::drop;

use piston::window::WindowSettings;
use piston::input::*;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;

use io::base::*;
use io::constants::*;
use io::utils::*;
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use entities::entity::{EntState, Entities, Ticks, do_actions, schedule_actions};
use entities::interact::{select_entities, add_dig_tasks, Tasks};
use entities::pathfind::path_to;


pub struct Game {
    pub gl: GlGraphics,
    pub input: Input,
    pub state: GameState,
    pub done: bool,
}


pub struct GameState {
    pub ch: CameraHandle,
    pub map: Map,
    pub ent_state: EntState,
    pub selected_entities: Vec<Id>,
    pub tasks: Tasks,
    pub ticks: Ticks,
    #[allow(dead_code)]
    pub cur_id: Id, // Global state for giving things ids
}


pub struct Input {
    mouse_pos: WinPos,
    pub selector: Option<Selector>,
    pub selector_start: Option<WinPos>, 
    pub sel_state: SelState,

}


impl GameState {
    // Contains all state corresponding to a running game
    pub fn new(map: Map, ent_state: EntState) -> GameState {
        GameState {
            ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
            map: map,
            ent_state: ent_state,
            selected_entities: Vec::new(),
            tasks: Vec::new(),
            ticks: 0,
            cur_id: 0,
        }
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.ticks += 1;

        // Entity update and pathfinding
        schedule_actions(self);
        do_actions(self);

        drop(args);
    }

    pub fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map)
    }

    #[allow(dead_code)]
    pub fn give_id(&mut self) -> Id {
        self.cur_id += 1;
        self.cur_id
    }
}


impl Game {
    // Top level global state
    pub fn new(map: Map, ent_state: EntState) -> Game {
        Game {
            gl: GlGraphics::new(OPEN_GL_VERSION),
            input: Input::new(),
            state: GameState::new(map, ent_state),
            done: false,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        // TODO Keep track of FPS 
        // TODO Dynamically resize window bounds

        let snap = self.state.get_snap();
        let ent_state = &self.state.ent_state;
        let ch = &self.state.ch;
        let map = &self.state.map;
        let selector = self.input.selector;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            draw_tiles(c, gl, &snap, map);
            draw_entities(c, gl, ch, ent_state);
            draw_selector(c, gl, selector);
        });
    }

    pub fn forward(&mut self) {
        self.state.ch.y -= 1;
    }

    pub fn back(&mut self) {
        self.state.ch.y += 1;
    }

    pub fn left(&mut self) {
        self.state.ch.x -= 1;
    }

    pub fn right(&mut self) {
        self.state.ch.x += 1;
    }

    pub fn up(&mut self) {
        self.state.ch.z -= 1;
    }

    pub fn down(&mut self) {
        self.state.ch.z += 1;
    }

    pub fn quit(&mut self) {
        self.done = true;
    }

    pub fn set_digging(&mut self) {
        self.input.sel_state = SelState::Digging;
    }

    pub fn move_to(&mut self) {
        let mouse_pos = self.input.mouse_pos;
        self.move_selected_entities(mouse_pos);
    }

    pub fn null(&mut self) {}

    pub fn press_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            self.input.selector_start = Some(self.input.mouse_pos);
            self.input.selector = Some((self.input.mouse_pos, self.input.mouse_pos))
        }

        // TODO Change to dictionary mapping keys to functions
        if let Button::Keyboard(key) = button {
            let func = match key {
                Key::Right  => Game::right,
                Key::Left   => Game::left, 
                Key::Down   => Game::back,
                Key::Up     => Game::forward, 
                Key::Comma  => Game::down,
                Key::Period => Game::up,
                Key::D      => Game::set_digging,
                Key::H      => Game::left,
                Key::J      => Game::back,
                Key::K      => Game::forward,
                Key::L      => Game::right,
                Key::O      => Game::up, 
                Key::P      => Game::down, 
                Key::Q      => Game::quit,
                Key::Y      => Game::move_to,
                _           => Game::null,
            };

            func(self);
        }
    }

    fn move_selected_entities(&mut self, mouse_pos: WinPos) {
        let dest_tile_pos = win_pos_to_tile(mouse_pos, &self.state.ch);

        for ref mut ent in &mut self.state.ent_state.entities {
            for ent_id in &self.state.selected_entities {
                if ent.id == *ent_id {
                    ent.actions = path_to(&self.state.map, ent, dest_tile_pos);
                }
            }
        }

        self.state.selected_entities.clear();
    }
}


impl Input {
    fn new() -> Input {
        Input {
            mouse_pos: (0.0, 0.0),
            selector: None,
            sel_state: SelState::Ents,
            selector_start: None,
        }
    }

    pub fn release_button(&mut self, state: &mut GameState, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            if let Some(selector) = self.selector {   
                let tiles_selector = win_to_tile_selector(selector, &state.ch);

                if self.sel_state == SelState::Ents {
                    state.selected_entities = select_entities(&state.ent_state.entities, 
                                                              tiles_selector);
                } else {
                    add_dig_tasks(&mut state.tasks, &mut state.map, tiles_selector);
                    self.sel_state = SelState::Ents;
                }

                self.selector_start = None;
                self.selector = None;
            }
        }
    }


    pub fn move_cursor(&mut self, pos: [f64; 2]) {
        self.mouse_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.selector_start {
            self.selector = Some((selector_pos, self.mouse_pos));
        }
    }
}


fn draw_tiles(c: Context, gl: &mut GlGraphics, 
              snap: &MapSnapshot, map: &Map) {
    let square = rectangle::square(0.0, 0.0, X_PIXELS);

    for y in 0..snap.ylen {
        for x in 0..snap.xlen {
            let index = (x + y * snap.xlen) as usize;
            let tile = snap.tiles[index];
            let xpos = X_PIXELS * (x as f64);
            let ypos = Y_PIXELS * (y as f64);
            let transform = c.transform.trans(xpos, ypos);
            let color = match map.materials.get(&tile.material) {
                Some(material) => {
                    material.color
                },
                None => BLACK,
            };
            rectangle(color, square, transform, gl);
        }
    }
}

fn draw_entities(c: Context, gl: &mut GlGraphics, 
                 ch: &CameraHandle, ent_state: &EntState) {
    let square = rectangle::square(0.0, 0.0, X_PIXELS);

    for entity in ent_state.entities.iter() {
        let (x, y, z) = entity.pos;
        if z == ch.z &&
               (ch.x <= x) && (x <= ch.xlen) &&
               (ch.y <= y) && (y <= ch.ylen) {
            let (winx, winy) = tile_pos_to_win(entity.pos, &ch);
            let transform = c.transform.trans(winx, winy);
            rectangle(YELLOW, square, transform, gl);
        }
    }
}

fn draw_selector(c: Context, gl: &mut GlGraphics, selector: Option<Selector>) {
    if let Some(((x1, y1), (x2, y2))) = selector {
        let selector_rect = [x1, y1, x2 - x1, y2 - y1];
        rectangle(SELECTOR_COLOR, selector_rect, c.transform, gl);
    }
}

pub fn init_game(map: Map, ent_state: EntState) -> Game {
    Game::new(map, ent_state)
}

pub fn init_graphics() -> Window {
    WindowSettings::new(
        "SpaceFort",
        [X_WIN_SIZE, Y_WIN_SIZE]
        )
        .opengl(OPEN_GL_VERSION)
        .exit_on_esc(true)
        .build()
        .unwrap()
}
