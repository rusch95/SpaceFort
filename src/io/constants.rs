use opengl_graphics::OpenGL;

pub const OPEN_GL_VERSION: OpenGL = OpenGL::V3_2;

pub type Color = [f32; 4];
pub const BLACK:   Color = [0.0, 0.0, 0.0, 1.0];
#[allow(dead_code)]
pub const WHITE:   Color = [1.0, 1.0, 1.0, 1.0];
pub const YELLOW:   Color = [0.5, 0.5, 1.0, 1.0];
#[allow(dead_code)]
pub const RED:   Color = [1.0, 0.0, 0.0, 1.0];
#[allow(dead_code)]
pub const BLUE:   Color = [0.0, 0.0, 1.0, 1.0];
#[allow(dead_code)]
pub const GREEN:   Color = [0.0, 1.0, 0.0, 1.0];
pub const SELECTOR_COLOR: Color = [0.54, 0.69, 0.93, 0.5];

pub const X_WIN_SIZE: u32 = 800; 
pub const Y_WIN_SIZE: u32 = 800;
pub const X_NUM_TILES: i32 = 50;
pub const Y_NUM_TILES: i32 = 50;
pub const X_PIXELS: f64 = (X_WIN_SIZE / (X_NUM_TILES as u32)) as f64;
pub const Y_PIXELS: f64 = (Y_WIN_SIZE / (Y_NUM_TILES as u32)) as f64;
