extern crate ncurses;

use ncurses::*;
use std::sync::{RwLock, Arc};
use io::base::CameraHandle;
use map::tiles::{Map, MapSnapshot, Tile, handle_to_snapshot};


pub fn init_graphics(map: Arc<RwLock<Map>>) {
    let handle = init_term();
    loop {
        term_handle.update_display(&map);
        if !term_handle.get_input() {
            break;
        }
    }
    end_term();

pub fn init_term() -> CameraHandle {
    //Initialize ncurses terminal and setup Camera Handle
    initscr();
    raw();

    keypad(stdscr(), true);
    noecho();

	curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);

    //Set getch to be non-blocking
    timeout(0);

    CameraHandle {xlen: 60, ylen: 30, x: 0, y: 0, z: 0}
}

pub fn end_term() {
    //Tear down for ncurses terminal
    echo();
    curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
    endwin();
}

impl CameraHandle {

    pub fn update_display(&self, map: Arc<RwLock<Map>>) {
        //Update screen based off changes to map, creatures, and such
        let snap = handle_to_snapshot(self, map.read().unwrap());
        for y in 0..snap.ylen {
            for x in 0..snap.xlen {
                let index = (x + y * snap.xlen) as usize;
                update_tile(x, y, snap.tiles[index])
            }
        }
        refresh();
    }

    #[allow(dead_code)]
    pub fn test_snap(&self) -> MapSnapshot {
        //Create MapSnapshot for testing purposes 
        //[debug] func
        let mut tiles = Vec::new();
        for y in self.y..self.y + self.ylen {
            for x in self.x..self.x + self.xlen {
                let tile = Tile {material: (x + y * 10) as u16};
                tiles.push(tile)
            }
        }
        MapSnapshot {tiles: tiles, xlen: self.xlen, ylen: self.ylen}
    }

    pub fn get_input(&mut self) -> bool {
        //Get keyboard input, updating CameraHandle, and changing map as necessary
        //TODO Enable key bindings
        //TODO Allow char instead of raw ascii 
		let ch = getch();
        match ch
        {
          KEY_LEFT  => {self.x -= 1;},
          KEY_RIGHT => {self.x += 1;},
          KEY_UP    => {self.y -= 1;},
          KEY_DOWN  => {self.y += 1;},
          60        => {self.z -= 1;},
          62        => {self.z += 1;},
          81 =>
          {
              end_term();      
              return false;
          },
          _ => { }

        }

        //Debuging info
        mvprintw(50, 5, &format!("CameraHandle x:{}, y:{}, z:{}", self.x, self.y, self.z));

        true
    }
}

fn update_tile(x: i32, y: i32, tile: Tile) {
    //Write tile to screen handling color and character
    if tile.material == 60000u16 {
        mvaddch(y, x, 33);
    } else {
        mvprintw(y, x, &format!("{}", tile.material % 10));
    }
}
