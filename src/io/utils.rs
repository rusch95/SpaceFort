use io::base::*;
use io::constants::*;
use game::base::*;


/// Convert a window position (x, y) to its corresponding tile position (x, y, z)
pub fn win_pos_to_tile(win_pos: WinPos, ch: &CameraHandle) -> Pos {
    let (x, y) = win_pos;
    ((x / X_PIXELS) as i32 + ch.x, 
     (y / Y_PIXELS) as i32 + ch.y, 
      ch.z)
}

/// Convert a tile position (x, y, z) to a location on the graphical window (x, y)
pub fn tile_pos_to_win(pos: Pos, ch: &CameraHandle) -> WinPos {
    let (x, y, _) = pos;
    (f64::from(x - ch.x) * X_PIXELS,
     f64::from(y - ch.y) * Y_PIXELS)
}

/// Convert a 2D window mouse selection to a tile selection
pub fn win_to_tile_selector(selector: Selector, ch: &CameraHandle) -> TilesSelector {
    let (win_pos1, win_pos2) = selector;
    (win_pos_to_tile(win_pos1, ch), win_pos_to_tile(win_pos2, ch))
}

pub fn sel_dist(selector: Selector) -> f64 {
    let ((x1, y1), (x2, y2)) = selector;
    ((x1 - x2).powf(2.0) + (y1 - y2).powf(2.0))
}
