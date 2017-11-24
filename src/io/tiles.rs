use piston::window::WindowSettings;
use piston::input::*;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;

use io::base::*;
use io::constants::*;
use io::utils::*;
use game::base::{Game, GameState};
use map::tiles::{Map, MapSnapshot};
use entities::entity::Entities;
use entities::interact::{select_entities, add_dig_tasks};


pub struct Input {
    pub mouse_pos: WinPos,
    pub selector: Option<Selector>,
    pub selector_start: Option<WinPos>, 
    pub sel_state: SelState,

}


impl Input {
    pub fn new() -> Input {
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
                    state.selected_entities = select_entities(&state.entities, 
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
            let xpos = X_PIXELS * f64::from(x);
            let ypos = Y_PIXELS * f64::from(y);
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
            let (winx, winy) = tile_pos_to_win(entity.pos, ch);
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


pub fn render(game: &mut Game, args: &RenderArgs) {
    // TODO Keep track of FPS 
    // TODO Dynamically resize window bounds

    let snap = game.state.get_snap();
    let entities = &game.state.entities;
    let ch = &game.state.ch;
    let map = &game.state.map;
    let selector = game.input.selector;

    game.gl.draw(args.viewport(), |c, gl| {
        // Clear the screen.
        clear(BLACK, gl);

        draw_tiles(c, gl, &snap, map);
        draw_entities(c, gl, ch, entities);
        draw_selector(c, gl, selector);
    });
}
