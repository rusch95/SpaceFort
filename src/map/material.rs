use std::path::Path;
use map::constants::*;

type MaterialID = u16;
type Materials = Vec<Material>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Material {
    // The Stuff dreams are made of
    pub material: MaterialID,
    pub diggable: bool,
    pub color: [f32; 4],
}

pub fn init_materials(root: &Path) {
    
}
