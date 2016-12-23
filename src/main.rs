extern crate ncurses;

use ncurses::*;

fn main()
{
  /* Setup ncurses. */
  initscr();
  raw();

  /* Allow for extended keyboard (like F1). */
  keypad(stdscr(), true);
  noecho();

  /* Invisible cursor. */
  curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);


  endwin();
}

