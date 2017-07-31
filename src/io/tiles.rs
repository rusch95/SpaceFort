use std::sync::{RwLock, Arc};
use io::base::CameraHandle;
use map::tiles::{Map, MapSnapshot, Tile, handle_to_snapshot};

use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::{ GlGraphics, OpenGL };


pub struct App {
    gl: GlGraphics,
    ch: CameraHandle,
    map: Arc<RwLock<Map>>,
    pos: (f64, f64)
}


impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED:   [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLUE:  [f32; 4] = [0.0, 0.0, 1.0, 1.0];

        let square = rectangle::square(0.0, 0.0, 20.0);
        let (x, y) = self.pos;

        let snap = self.get_snap();

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(GREEN, gl);

            for y in 0..snap.ylen {
                for x in 0..snap.xlen {
                    let index = (x + y * snap.xlen) as usize;
                    let tile = snap.tiles[index];
                    let xpos = 20.0 * (x as f64);
                    let ypos = 20.0 * (y as f64);
                    let transform = c.transform.trans(xpos, ypos);
                    let color = match tile.material {1 => BLUE, 60000 => GREEN, _ => RED};
                    rectangle(color, square, transform, gl);
                }
            }

            // Draw a box rotating around the middle of the screen.
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
    }

    fn get_snap(&mut self) -> MapSnapshot {
        handle_to_snapshot(&self.ch, &self.map.read().unwrap())
    }
}


pub fn init_graphics(map: Arc<RwLock<Map>>) {
    let opengl = OpenGL::V3_2;

    let mut window: Window = WindowSettings::new(
        "SpaceFort",
        [600, 600]
        )
        .opengl(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let mut app = App {
        gl: GlGraphics::new(opengl),
        ch: CameraHandle {xlen: 30, ylen: 30, x: 0, y: 0, z: 0},
        map: map,
        pos: (0.0, 0.0)
    };

    let mut cursor = (0.0, 0.0);

    let mut events = Events::new(EventSettings::new()).lazy(true);
    while let Some(e) = events.next(&mut window) {

        e.mouse_cursor(|x, y| {
            cursor = (x, y);
            println!("Mouse moved '{} {}'", x, y);
        });

        if let Some(Button::Mouse(button)) = e.press_args() {
            println!("Pressed mouse button '{:?}'", button);
            app.pos = cursor;
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            match key {
                Key::Right => app.ch.x += 1,
                Key::Left  => app.ch.x -= 1,
                Key::Down  => app.ch.y += 1,
                Key::Up    => app.ch.y -= 1,
                _          => {},
            }

            println!("Pressed keyboard key '{:?}'", key);
        };

        if let Some(r) = e.render_args() {
            app.render(&r);
        }

        if let Some(u) = e.update_args() {
            app.update(&u);
        }
    }
}

impl CameraHandle {

    pub fn update_display(&self, map: &Map) {
        //Update screen based off changes to map, creatures, and such
        let snap = handle_to_snapshot(self, map);
    }
}
