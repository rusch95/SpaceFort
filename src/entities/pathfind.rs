use map::tiles::Map;
use entities::interact::{Action, Actions, ActionType};
use entities::entity::{Entity, Pos};

use pathfinding::{bfs, fringe};

pub fn path_to(map: &Map, ent: &mut Entity, pos: Pos) {
    let mut actions = Actions::new();
    let path = fringe(&ent.pos,
                      |&p| succ(&map, &p),
                      |&p| dist(&p, &pos),
                      |&p| p == pos
                     );
    match path {
        Some(path) => {
            for coord in path {
                actions.push_back(Action { atype: ActionType::Move(coord),
                                           duration: ent.movement_speed });
            }
        },
        None => (),
    }

    ent.actions = actions;
}

            let (path, cost) = path;
            for coord in path {
                actions.push_back(Action { atype: ActionType::Move(coord),
                                           duration: ent.movement_speed });  // TODO Swap out for ent speed
            }
        },
        None => (),
    }

    actions
}

const SQRT2: f64 = 1.41;

// TODO Rewrite using filter
fn succ(map: &Map, pos: &Pos) -> Vec<(Pos, f64)> {
    let (x, y, z) = *pos;

    // TODO Make this a const instead
    let mut poss = Vec::new();
    for i in [-1, 0, 1].iter() {
        for j in [-1, 0, 1].iter() {
            let cost = {if i == 0 or j == 0 {1.0} else {SQRT2}};
            poss.push(((x + i, y + j, z), cost));
        }
    };

    let mut vec = Vec::new();
    for ((i, j, k), cost) in poss {
        // 2d movement
        if let Some(tile) = map.get_tile(i, j, k) {
            match tile.material {
                0 => vec.push(((i, j, k), cost)),
                8 => vec.push(((i, j, k), cost)),
                9 => vec.push(((i, j, k), cost)),
                _ => (),
            }
        }
        // Up and down
        if let Some(tile) = map.get_tile(x, y, z) {
            match tile.material {
                8 => vec.push(((x, y, z - 1), 1.0)),
                9 => vec.push(((x, y, z + 1), 1.0)),
                _ => (),
            }
        }
    }

    vec
 }
