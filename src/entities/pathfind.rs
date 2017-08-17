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

    // TODO Make this a const instead
    let mut poss = Vec::new();
    for i in [-1, 0, 1].iter() {
        for j in [-1, 0, 1].iter() {
            poss.push((x + i, y + j, z));
        }
    };

    let mut vec = Vec::new();
    for (i, j, k) in poss {
        // 2d movement
        if let Some(tile) = map.get_tile(i, j, k) {
            match tile.material {
                0 => vec.push((i, j, k)),
                8 => vec.push((i, j, k)),
                9 => vec.push((i, j, k)),
                _ => (),
            }
        }
        // Up and down
        if let Some(tile) = map.get_tile(x, y, z) {
            match tile.material {
                8 => vec.push((x, y, z - 1)),
                9 => vec.push((x, y, z + 1)),
                _ => (),
            }
        }
    }

    vec
 }
