use std::collections::HashSet;

use piston::window::WindowSettings;
use piston::input::*;
use graphics::*;
use glutin_window::GlutinWindow as Window;
use opengl_graphics::GlGraphics;

use io::base::*;
use io::constants::*;
use io::textures::*;
use io::utils::*;
use game::base::*;
use game::client::Client;
use map::tiles::{Map, MapSnapshot};
use entities::creatures::{CreatureMap, get_color};
use entities::entity::{Entity, EntID, EntIDs};


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

pub fn render(player: &mut Client, args: &RenderArgs) {
    // TODO Keep track of FPS 
    // TODO Dynamically resize window bounds

    let snap = player.get_snap();
    let map = &player.map;
    let entities = &mut player.entities;
    let ch = &player.ch;
    let selector = player.selector;
    let creature_types = &player.creature_types;
    let gl = &mut player.gl;
    let selected_ents = &player.selected_entities;
    let textures = &player.textures;

    gl.draw(args.viewport(), |c, gl| {
        // Clear the screen.
        clear(BLACK, gl);

        draw_tiles(c, gl, &snap, map, textures);
        draw_entities(c, gl, ch, entities, creature_types, selected_ents);
        draw_selector(c, gl, selector);
    });
}

fn draw_tiles(c: Context, gl: &mut GlGraphics, snap: &MapSnapshot, 
              map: &Map, textures: &Textures) {
    let square = rectangle::square(0.0, 0.0, X_PIXELS);

    for y in 0..snap.ylen {
        for x in 0..snap.xlen {
            let index = (x + y * snap.xlen) as usize;
            let tile = snap.tiles[index];
            let xpos = X_PIXELS * f64::from(x);
            let ypos = Y_PIXELS * f64::from(y);
            let transform = c.transform.trans(xpos, ypos);
            let color = match map.materials.get(&tile.material) {
                Some(material) => { material.color },
                None => BLACK,
            };
            if let Some(texture) = textures.get(&tile.material) {
                Image::new()
                    .rect(rectangle::square(0.0, 0.0, X_PIXELS))
                    .draw(texture, &c.draw_state, transform, gl);
            } else {
                rectangle(color, square, transform, gl);
            }
        }
    }
}

fn draw_entities(c: Context, gl: &mut GlGraphics, ch: &CameraHandle, 
                 entities: &[Entity], creature_types: &CreatureMap, 
                 selected_ents: &EntIDs) {
    // Scale entities slightly smaller than a square for now
    let inner_square = rectangle::square(0.0, 0.0, X_PIXELS * 0.85);
    let outer_square = rectangle::square(0.0, 0.0, X_PIXELS * 1.0);

    // Transform EntIDs into map
    let sel_ents_set: HashSet<EntID> = selected_ents.iter().cloned().collect();

    for ent in entities {
        if ch.in_bounds(&ent.pos) {
            let (winx, winy) = tile_pos_to_win(ent.pos, ch);
            let inner_transform = c.transform.trans(winx + X_PIXELS * 0.075, 
                                                    winy + X_PIXELS * 0.075);
            let outer_transform = c.transform.trans(winx, winy);

            // Draw team color outline or selection
            let outline_color = if sel_ents_set.contains(&ent.id) { WHITE }
                                else { team_color(ent.team_id) };
            rectangle(outline_color, outer_square, outer_transform, gl);

            // Draw color of ent
            let ent_color = if ent.alive { 
                get_color(&ent.creature_id, creature_types)
            } else {
                BLACK
            };
            rectangle(ent_color, inner_square, inner_transform, gl);
        }
    }
}

/// Outline each entity in it's team's color
fn team_color(team_id: TeamID) -> Color {
    match team_id {
        Some(1) => BLUE,
        Some(2) => RED,
        _       => WHITE,
    }
}
        
/// Draw the current mouse selection
fn draw_selector(c: Context, gl: &mut GlGraphics, selector: Option<Selector>) {
    if let Some(((x1, y1), (x2, y2))) = selector {
        let selector_rect = [x1, y1, x2 - x1, y2 - y1];
        rectangle(SELECTOR_COLOR, selector_rect, c.transform, gl);
    }
}
