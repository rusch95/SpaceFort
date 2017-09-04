use std::mem::drop;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

use io::base::*;
use io::constants::*;
use io::utils::*;
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use entities::entity::{Entities, Ticks, do_actions, schedule_actions};
use entities::interact::{select_entities, add_dig_tasks, Tasks};
use entities::pathfind::path_to;


pub struct Game {
    pub gl: GlGraphics,

    pub ch: CameraHandle,
    pub map: Map,

    pub entities: Entities,
    pub tasks: Tasks,

    cur_pos: WinPos,

    pub selected_entities: Vec<Id>,
    pub selector: Option<Selector>,
    pub selector_start: Option<WinPos>, 
    pub sel_state: SelState,

    pub ticks: Ticks,
    #[allow(dead_code)]
    pub cur_id: Id, // Global state for giving things ids

    pub done: bool,
}

pub struct Graphics {
}


impl Game {

    fn new(opengl: OpenGL, map: Map, entities: Entities) -> Game {
        Game {
                gl: GlGraphics::new(opengl),
                ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
                map: map,
                entities: entities,
                tasks: Vec::new(),
                selected_entities: Vec::new(),
                selector: None,
                ticks: 0,
                cur_id: 0,
                cur_pos: (0.0, 0.0),
                sel_state: SelState::Ents,
                selector_start: None,
                done: false,
            }
    }

    fn render(&mut self, args: &RenderArgs) {
        let snap = self.get_snap();
        let entities = &self.entities;
        let ch = &self.ch;
        let map = &self.map;
        let selector = self.selector;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            draw_tiles(c, gl, &snap, map);
            draw_entities(c, gl, ch, entities);
            draw_selector(c, gl, selector);
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.ticks += 1;

        // Entity update and pathfinding
        schedule_actions(self);
        do_actions(self);

        drop(args);
    }

    fn press_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            self.selector_start = Some(self.cur_pos);
            self.selector = Some((self.cur_pos, self.cur_pos))
        }

        // TODO Change to dictionary mapping keys to functions
        if let Button::Keyboard(key) = button {
            match key {
                Key::Right => self.ch.x += 1,
                Key::Left  => self.ch.x -= 1,
                Key::Down  => self.ch.y += 1,
                Key::Up    => self.ch.y -= 1,
                Key::O     => self.ch.z += 1,
                Key::P     => self.ch.z -= 1,
                Key::D     => self.sel_state = SelState::Digging,
                Key::Y     => {let cur_pos = self.cur_pos;
                               self.move_selected_entities(cur_pos);},
                Key::Q     => self.done = true,
                _          => (),
            }
        }
    }

    fn release_button(&mut self, button: Button) {
        if button == Button::Mouse(MouseButton::Left) {
            if let Some(selector) = self.selector {   
                let tiles_selector = win_to_tile_selector(selector, &self.ch);

                if self.sel_state == SelState::Ents {
                    self.selected_entities = select_entities(&self.entities, tiles_selector);
                } else {
                    add_dig_tasks(&mut self.tasks, &mut self.map, tiles_selector);
                    self.sel_state = SelState::Ents;
                }

                self.selector_start = None;
                self.selector = None;
            }
        }
    }

    fn move_cursor(&mut self, pos: [f64; 2]) {
        self.cur_pos = (pos[0], pos[1]);

        if let Some(selector_pos) = self.selector_start {
            self.selector = Some((selector_pos, self.cur_pos));
        }
    }

    #[allow(dead_code)]
    fn give_id(&mut self) -> Id {
        self.cur_id += 1;
        self.cur_id
    }

    fn move_selected_entities(&mut self, cur_pos: WinPos) {
        let dest_tile_pos = win_pos_to_tile(cur_pos, &self.ch);

        for ref mut ent in &mut self.entities {
            for ent_id in &self.selected_entities {
                if ent.id == *ent_id {
                    ent.actions = path_to(&self.map, ent, dest_tile_pos);
                }
            }
        }

        self.selected_entities.clear();
    }

    fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map)
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

pub fn init_graphics(map: Map, entities: Entities) {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        "SpaceFort",
        [X_WIN_SIZE, Y_WIN_SIZE]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut game = Game::new(opengl, map, entities);
        
    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(button) = e.press_args() {
            game.press_button(button);
        }

        if let Some(pos) = e.mouse_cursor_args() {
            game.move_cursor(pos);
        }

        if let Some(button) = e.release_args() {
            game.release_button(button);
        }

        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            game.update(&u);
        }

        if game.done {
            break;
        }
    }
}
