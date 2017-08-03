use std::sync::{RwLock, Arc};
use std::{thread, time};
use io::base::CameraHandle;
use map::tiles::Map;

// use entities::interact::Action;

pub struct Entity {
    pos: (u16, u16),
    // actions: Vec<Action>,
}

 
pub fn init_entities(map: Arc<RwLock<Map>>) {

    thread::spawn(move || {
        loop {
        }
    });
}
