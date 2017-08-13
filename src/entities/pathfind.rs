use map::tiles::Map;
use entities::interact::{Action, Actions, ActionType};
use entities::entity::{Entity, Pos};

use pathfinding::bfs;

pub fn path_to(map: &Map, ent: &mut Entity, pos: Pos) {
    let mut actions = Actions::new();
    let path = bfs(&ent.pos,
                   |&(x, y, z)| succ(&map, &(x, y, z)),
                   |&p| p == pos
                  );
    match path {
        Some(path) => {
            for coord in path {
                actions.push_back(Action { atype: ActionType::Move(coord),
                                           duration: 20 });
            }
        },
        None => (),
    }

    ent.actions = actions;
}

// TODO Rewrite using filter
fn succ(map: &Map, pos: &Pos) -> Vec<Pos> {
    let (x, y, z) = *pos;
    let mut vec = Vec::new();
    for (x,y,z) in vec![(x+1,y,z),(x-1,y,z),(x,y+1,z),(x,y-1,z),
                        (x+1,y+1,z),(x-1,y-1,z),(x+1,y-1,z),(x-1,y+1,z)] {
        if let Some(tile) = map.get_tile(x,y,z) {
            if tile.material == 0 {
                vec.push((x,y,z));
            }
        }
    }
    vec
 }
