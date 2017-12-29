use ncurses::*;
use map::tiles::{Map, Tile};
use game::term_client::TermClient;


pub fn init_term() {
    //Initialize ncurses terminal and setup Terminal Handle
    initscr();
    raw();

    keypad(stdscr(), true);
    noecho();

	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    //Set getch to be non-blocking
    timeout(0);
}

pub fn end_term() {
    //Tear down for ncurses terminal
    echo();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    endwin();
}

pub fn render(term: &mut TermClient, map: &Map) {
    //Update screen based off changes to map, creatures, and such
    let snap = term.get_snap();
    for y in 0..snap.ylen {
        for x in 0..snap.xlen {
            let index = (x + y * snap.xlen) as usize;
            update_tile(x, y, snap.tiles[index])
        }
    }
    refresh();
}

fn update_tile(x: i32, y: i32, tile: Tile) {
    //Write tile to screen handling color and character
    if tile.material == 60000u16 {
        mvaddch(y, x, 33);
    } else {
        mvprintw(y, x, &format!("{}", tile.material % 10));
    }
}
