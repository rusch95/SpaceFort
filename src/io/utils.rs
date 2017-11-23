use io::base::*;
use io::constants::*;
use entities::entity::Pos;


pub fn win_pos_to_tile(win_pos: WinPos, ch: &CameraHandle) -> Pos {
    let (x, y) = win_pos;
    ((x / X_PIXELS) as i32 + ch.x, 
     (y / Y_PIXELS) as i32 + ch.y, 
      ch.z)
}


pub fn tile_pos_to_win(pos: Pos, ch: &CameraHandle) -> WinPos {
    let (x, y, _) = pos;
    (f64::from(x - ch.x) * X_PIXELS,
     f64::from(y - ch.y) * Y_PIXELS)
}


pub fn win_to_tile_selector(selector: Selector, ch: &CameraHandle) -> TilesSelector {
    let (win_pos1, win_pos2) = selector;
    (win_pos_to_tile(win_pos1, ch), win_pos_to_tile(win_pos2, ch))
}
