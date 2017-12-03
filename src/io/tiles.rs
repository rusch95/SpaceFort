use piston::window::WindowSettings;
use piston::input::*;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;

use io::base::*;
use io::constants::*;
use io::utils::*;
use game::base::{GameState, Player, TeamID};
use map::tiles::{Map, MapSnapshot};
use entities::creatures::{CreatureMap, get_color};
use entities::entity::Entities;

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
                 ch: &CameraHandle, entities: &Entities,
                 creature_types: &CreatureMap) {
    // Scale entities slightly smaller than a square for now
    let inner_square = rectangle::square(0.0, 0.0, X_PIXELS * 0.85);
    let outer_square = rectangle::square(0.0, 0.0, X_PIXELS * 1.0);

    for ent in entities.iter() {
        let (x, y, z) = ent.pos;
        if z == ch.z &&
               (ch.x <= x) && (x <= ch.x + ch.xlen) &&
               (ch.y <= y) && (y <= ch.y + ch.ylen) {
            let (winx, winy) = tile_pos_to_win(ent.pos, ch);
            let inner_transform = c.transform.trans(winx + X_PIXELS * 0.075, 
                                                    winy + X_PIXELS * 0.075);
            let outer_transform = c.transform.trans(winx, winy);
            rectangle(team_color(ent.team_id), outer_square, outer_transform, gl);
            if ent.alive {
                let color = get_color(&ent.creature_id, creature_types);
                rectangle(color, inner_square, inner_transform, gl);
            } else {
                rectangle(BLACK, inner_square, inner_transform, gl);
            }
        }
    }
}

fn team_color(team_id: TeamID) -> Color {
    match team_id {
        Some(1) => BLUE,
        Some(2) => RED,
        _       => WHITE,
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

pub fn render(player: &mut Player, g_state: &mut GameState, args: &RenderArgs) {
    // TODO Keep track of FPS 
    // TODO Dynamically resize window bounds

    let entities = &g_state.entities;
    let map = &g_state.map;
    let snap = player.get_snap(map);
    let ch = &player.ch;
    let selector = player.selector;
    let creature_types = &g_state.creature_types;
    let gl = &mut player.gl;

    gl.draw(args.viewport(), |c, gl| {
        // Clear the screen.
        clear(BLACK, gl);

        draw_tiles(c, gl, &snap, map);
        draw_entities(c, gl, ch, entities, creature_types);
        draw_selector(c, gl, selector);
    });
}
