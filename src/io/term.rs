use ncurses::*;
use map::tiles::{Map, Tile};
use game::term_client::TermClient;


/// Initialize ncurses terminal
pub fn init_term() {
    initscr();
    raw();

    keypad(stdscr(), true);
    noecho();

	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    // Set getch to be non-blocking
    timeout(0);
}

/// Tear down for ncurses terminal
pub fn end_term() {
    echo();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    endwin();
}

/// Update screen based off changes to map, creatures, and such
pub fn render(term: &mut TermClient) {
    draw_tiles(term);
    draw_ents(term);
    refresh();
}

fn draw_tiles(term: &mut TermClient) {
    let snap = term.get_snap();
    for y in 0..snap.ylen {
        for x in 0..snap.xlen {
            let index = (x + y * snap.xlen) as usize;
            update_tile(x, y, snap.tiles[index])
        }
    }
}

fn draw_ents(term: &mut TermClient) {
    for ent in term.entities.iter() {
        if term.ch.in_bounds(&ent.pos) {
            // TODO Add back in color
            let (ox, oy, _) = ent.pos;
            let (x, y) = (ox - term.ch.x, oy - term.ch.y);
            mvaddch(y, x, 64);
        }
    }
}

/// Write tile to screen handling color and character
fn update_tile(x: i32, y: i32, tile: Tile) {
    if tile.material == 10 {
        mvaddch(y, x, 32);
    } else {
        mvprintw(y, x, &format!("{}", tile.material % 10));
    }
}
