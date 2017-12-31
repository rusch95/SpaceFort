use std::path::Path;
use std::collections::HashMap;

use opengl_graphics::Texture;
use piston_window::TextureSettings;

use map::material::*;


pub type Textures = HashMap<MaterialID, Texture>;

pub fn load_textures() -> Textures {
    let mut textures = HashMap::new();
    let path = Path::new("../static/inc/textures/materials/grass.png");
    let settings = TextureSettings::new();
    let texture = Texture::from_path(path, &settings).unwrap();
    textures.insert(7, texture);
    textures
}
