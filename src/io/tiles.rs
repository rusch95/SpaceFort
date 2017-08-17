extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate pathfinding;

use std::path::Path;
use io::base::CameraHandle;
use io::colors::*;
use map::constants::*;
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use entities::entity::{Entities, EntId, Pos, Ticks};
use entities::interact::{Action, Actions, ActionType, select_entities};
use entities::pathfind::path_to;

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };

const X_WIN_SIZE: u32 = 600; 
const Y_WIN_SIZE: u32 = 600;
const X_NUM_TILES: i32 = 35;
const Y_NUM_TILES: i32 = 35;
const X_PIXELS: f64 = (X_WIN_SIZE / (X_NUM_TILES as u32)) as f64;
const Y_PIXELS: f64 = (Y_WIN_SIZE / (Y_NUM_TILES as u32)) as f64;

pub type WinPos = (f64, f64);
pub type Selector = (WinPos, WinPos);
pub type TilesSelector = (Pos, Pos);

pub struct Game {
    gl: GlGraphics,
    ch: CameraHandle,
    map: Map,
    entities: Entities,
    selected_entities: Vec<EntId>,
    selector: Option<Selector>,
    ticks: Ticks,
}


impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let square = rectangle::square(0.0, 0.0, X_PIXELS);

        let snap = self.get_snap();

        let entities = &self.entities;
        let ch = &self.ch;

        let selector = self.selector;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            // Draw tiles
            for y in 0..snap.ylen {
                for x in 0..snap.xlen {
                    let index = (x + y * snap.xlen) as usize;
                    let tile = snap.tiles[index];
                    let xpos = X_PIXELS * (x as f64);
                    let ypos = Y_PIXELS * (y as f64);
                    let transform = c.transform.trans(xpos, ypos);
                    let color = match tile.material {1 => BLUE, AIR_MAT => BLACK, _ => RED};
                    rectangle(color, square, transform, gl);
                }
            }

            // Draw entities
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

            // Draw selector
            if let Some(((x1, y1), (x2, y2))) = selector {
                let selector_rect = [x1, y1, x2 - x1, y2 - y1];
                rectangle(SELECTOR_COLOR, selector_rect, c.transform, gl);
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.ticks += 1;

        // Entity update and pathfinding
        self.do_actions();
    }

    fn do_actions(&mut self) {

        for mut ent in self.entities.iter_mut() {
            let pop = match ent.actions.front_mut() {
                Some(act) => {
                    if act.duration > 0 {act.duration -= 1; false}
                    else {
                        match act.atype {
                            ActionType::Move(pos) => {
                                ent.pos = pos;
                            },
                            ActionType::Wait => {},
                        };
                        true
                    }
                }
                None => (false),
            };

            if pop {ent.actions.pop_front();};
        };
    }

    fn move_selected_entities(&mut self, cur_pos: WinPos) {
        let dest_tile_pos = win_pos_to_tile(cur_pos, &self.ch);

        for ref mut ent in &mut self.entities {
            for ent_id in &self.selected_entities {
                if ent.id == *ent_id {
                    path_to(&self.map, ent, dest_tile_pos);
                }
            }
        }

        self.selected_entities.clear();
    }

    fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map)
    }
}

pub fn win_pos_to_tile(win_pos: WinPos, ch: &CameraHandle) -> Pos {
    let (x, y) = win_pos;
    ((x / X_PIXELS) as i32 + ch.x, 
     (y / Y_PIXELS) as i32 + ch.y, 
      ch.z)
}

pub fn tile_pos_to_win(pos: Pos, ch: &CameraHandle) -> WinPos {
    let (x, y, _) = pos;
    ((x - ch.x) as f64 * X_PIXELS,
     (y - ch.y) as f64 * Y_PIXELS)
}

pub fn win_to_tile_selector(selector: Selector, ch: &CameraHandle) -> TilesSelector {
    let (win_pos1, win_pos2) = selector;
    (win_pos_to_tile(win_pos1, &ch), win_pos_to_tile(win_pos2, &ch))
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

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 1},
        map: map,
        entities: entities,
        selected_entities: Vec::new(),
        selector: None,
        ticks: 0,
    };

    let assets = Path::new(env!("CARGO_MANIFEST_DIR"))
                 .join("static/inc/assets");

    let mut cur_pos: WinPos = (0.0, 0.0);
    let mut selector_start: Option<WinPos> = None;

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Right => game.ch.x += 1,
                Key::Left  => game.ch.x -= 1,
                Key::Down  => game.ch.y += 1,
                Key::Up    => game.ch.y -= 1,
                Key::O     => game.ch.z += 1,
                Key::P     => game.ch.z -= 1,
                Key::Y     => {
                    game.move_selected_entities(cur_pos);
                },
                Key::Q     => break,
                _          => {},
            }
        };

        if let Some(pos) = e.mouse_cursor_args() {
            cur_pos = (pos[0], pos[1] + 43.0);

            if let Some(selector_pos) = selector_start {
                game.selector = Some((selector_pos, cur_pos));
            }
        }

        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                selector_start = Some(cur_pos);
                game.selector = Some((cur_pos, cur_pos))
            }
        }

        if let Some(button) = e.release_args() {
            if button == Button::Mouse(MouseButton::Left) {
                if let Some(selector) = game.selector {   
                    let tiles_selector = win_to_tile_selector(selector, &game.ch);
                    game.selected_entities = select_entities(&game.entities, tiles_selector);
                    selector_start = None;
                    game.selector = None;
                }
            }
        }

        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            game.update(&u);
        }
    }
}
