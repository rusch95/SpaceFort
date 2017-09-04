use std::mem::drop;
use std::collections::HashMap;

use piston::window::WindowSettings;
use piston::input::*;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;

use io::base::*;
use io::constants::*;
use io::utils::*;
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use entities::entity::{Entities, Ticks, do_actions, schedule_actions};
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
    pub entities: Entities,
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
    pub fn new(map: Map, entities: Entities) -> GameState {
        GameState {
            ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
            map: map,
            entities: entities,
            selected_entities: Vec::new(),
            tasks: Vec::new(),
            ticks: 0,
            cur_id: 0,
        }
    }
}


impl Game {
    // Top level global state
    pub fn new(map: Map, entities: Entities) -> Game {
        Game {
            gl: GlGraphics::new(OPEN_GL_VERSION),
            input: Input::new(),
            state: GameState::new(map, entities),
            done: false,
        }
    }

    pub fn render(&mut self, args: &RenderArgs) {
        let snap = self.get_snap();
        let entities = &self.state.entities;
        let ch = &self.state.ch;
        let map = &self.state.map;
        let selector = self.input.selector;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            draw_tiles(c, gl, &snap, map);
            draw_entities(c, gl, ch, entities);
            draw_selector(c, gl, selector);
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        self.state.ticks += 1;

        // Entity update and pathfinding
        schedule_actions(self);
        do_actions(self);

        drop(args);
    }

    pub fn press_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            self.input.selector_start = Some(self.input.mouse_pos);
            self.input.selector = Some((self.input.mouse_pos, self.input.mouse_pos))
        }

        // TODO Change to dictionary mapping keys to functions
        if let Button::Keyboard(key) = button {
            match key {
                Key::Right => self.state.ch.x += 1,
                Key::Left  => self.state.ch.x -= 1,
                Key::Down  => self.state.ch.y += 1,
                Key::Up    => self.state.ch.y -= 1,
                Key::O     => self.state.ch.z += 1,
                Key::P     => self.state.ch.z -= 1,
                Key::D     => self.input.sel_state = SelState::Digging,
                Key::Y     => {let mouse_pos = self.input.mouse_pos;
                               self.move_selected_entities(mouse_pos);},
                Key::Q     => self.done = true,
                _          => (),
            }
        }
    }

    pub fn release_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            if let Some(selector) = self.input.selector {   
                let tiles_selector = win_to_tile_selector(selector, &self.state.ch);

                if self.input.sel_state == SelState::Ents {
                    self.state.selected_entities = select_entities(&self.state.entities, tiles_selector);
                } else {
                    add_dig_tasks(&mut self.state.tasks, &mut self.state.map, tiles_selector);
                    self.input.sel_state = SelState::Ents;
                }

                self.input.selector_start = None;
                self.input.selector = None;
            }
        }
    }

    pub fn move_cursor(&mut self, pos: [f64; 2]) {
        self.input.mouse_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.input.selector_start {
            self.input.selector = Some((selector_pos, self.input.mouse_pos));
        }
    }

    #[allow(dead_code)]
    pub fn give_id(&mut self) -> Id {
        self.state.cur_id += 1;
        self.state.cur_id
    }

    fn move_selected_entities(&mut self, mouse_pos: WinPos) {
        let dest_tile_pos = win_pos_to_tile(mouse_pos, &self.state.ch);

        for ref mut ent in &mut self.state.entities {
            for ent_id in &self.state.selected_entities {
                if ent.id == *ent_id {
                    ent.actions = path_to(&self.state.map, ent, dest_tile_pos);
                }
            }
        }

        self.state.selected_entities.clear();
    }

    fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.state.ch, &self.state.map)
    }
}


impl GameState {

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
                 ch: &CameraHandle, entities: &Entities) {
    let square = rectangle::square(0.0, 0.0, X_PIXELS);

    for entity in entities.iter() {
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

pub fn init_game(map: Map, entities: Entities) -> Game {
    Game::new(map, entities)
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
