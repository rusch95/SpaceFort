pub const BLACK:   [f32; 4] = [0.0, 0.0, 0.0, 1.0];
#[allow(dead_code)]
pub const WHITE:   [f32; 4] = [1.0, 1.0, 1.0, 1.0];
pub const YELLOW:   [f32; 4] = [0.0, 1.0, 1.0, 1.0];
pub const SELECTOR_COLOR: [f32; 4] = [0.54, 0.69, 0.93, 0.5];

pub const X_WIN_SIZE: u32 = 600; 
pub const Y_WIN_SIZE: u32 = 600;
pub const X_NUM_TILES: i32 = 35;
pub const Y_NUM_TILES: i32 = 35;
pub const X_PIXELS: f64 = (X_WIN_SIZE / (X_NUM_TILES as u32)) as f64;
pub const Y_PIXELS: f64 = (Y_WIN_SIZE / (Y_NUM_TILES as u32)) as f64;
