extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate pathfinding;

use std::rc::Rc;
use std::path::Path;
use io::base::CameraHandle;
use io::colors::*;
use map::constants::*;
use map::tiles::{Map, MapSnapshot, handle_to_snapshot};
use entities::entity::{Entity, Entities, Pos, Ticks};
use entities::interact::{Action, Actions, ActionType};
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

type WinPos = (f64, f64);

pub struct Game {
    gl: GlGraphics,
    ch: CameraHandle,
    map: Map,
    entities: Entities,
    ticks: Ticks,
}


impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let square = rectangle::square(0.0, 0.0, X_PIXELS);

        let snap = self.get_snap();

        let entities = &self.entities;
        let ch = &self.ch;

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
                    let xpos = X_PIXELS * ((x - ch.x) as f64);
                    let ypos = Y_PIXELS * ((y - ch.y) as f64);
                    let transform = c.transform.trans(xpos, ypos);
                    rectangle(YELLOW, square, transform, gl);
                }
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

    fn win_pos_to_tile(&self, pos: WinPos) -> Pos {
        let (x, y) = pos;
        ((x / X_PIXELS) as i32, 
         (y / Y_PIXELS) as i32, 
          self.ch.z)
    }

    fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map)
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

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        ch: CameraHandle {xlen: X_NUM_TILES, ylen: Y_NUM_TILES, x: 0, y: 0, z: 0},
        map: map,
        entities: entities,
        ticks: 0,
    };

    let assets = Path::new(env!("CARGO_MANIFEST_DIR"))
                 .join("static/inc/assets");

    let mut cur_pos: WinPos = (0.0, 0.0);

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
                Key::Q     => break,
                _          => {},
            }
        };

        if let Some(pos) = e.mouse_cursor_args() {
            cur_pos = (pos[0], pos[1]);
        }

        if let Some(button) = e.press_args() {
            if button == Button::Mouse(MouseButton::Left) {
                let tile_pos = game.win_pos_to_tile(cur_pos);
                let ref mut ent = &mut game.entities[0];
                path_to(&game.map, ent, tile_pos);
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

