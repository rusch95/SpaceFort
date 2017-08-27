use map::tiles::Map;
use entities::interact::{Action, Actions, ActionType};
use entities::entity::{Entity, Pos};

use pathfinding::fringe;

// TODO Fix distances to use f64 instead of int

fn dist(pos1: &Pos, pos2: &Pos) -> i32 {
    let (x1, y1, z1) = *pos1;
    let (x2, y2, z2) = *pos2;
    let sqr_dist = (x1 - x2).pow(2) + (y1 - y2).pow(2) + (z1 - z2).pow(2);
    (sqr_dist as f64).sqrt() as i32
}

pub fn path_to(map: &Map, ent: &mut Entity, end_pos: Pos) -> Actions {
    let mut actions = Actions::new();
    let path = fringe(&ent.pos,
                      |&p| succ(&map, &p),
                      |&p| dist(&p, &end_pos),
                      |&p| p == end_pos
                     );
    match path {
        Some(path) => {
            let (path, cost) = path;
            for coord in path {
                actions.push_back(
                    Action { atype: ActionType::Move(coord),
                             duration: ent.movement_speed }
                );
            }
        },
        None => (),
    }

    actions
}

pub fn path_next_to(map: &Map, ent: &Entity, end_pos: Pos) -> Actions {
    let mut actions = Actions::new();
    let path = fringe(&ent.pos,
                      |&p| succ(&map, &p),
                      |&p| dist(&p, &end_pos),
                      |&p| {let sucs = succ(&map, &end_pos);
                                sucs.contains(&(p, UNIT_DIST)) || 
                                sucs.contains(&(p, DIAG_DIST))
                           }
                     );
    match path {
        Some(path) => {
            let (path, cost) = path;
            for coord in path {
                actions.push_back(
                    Action { atype: ActionType::Move(coord),
                             duration: ent.movement_speed }
                );
            }
        },
        None => (),
    }

    actions
}

const UNIT_DIST: i32 = 100;
const DIAG_DIST: i32 = (UNIT_DIST as f64 * 1.414) as i32;

// TODO Rewrite using filter
fn succ(map: &Map, pos: &Pos) -> Vec<(Pos, i32)> {
    let (x, y, z) = *pos;

    // TODO Make this a const instead
    let mut poss = Vec::new();
    for i in [-1, 0, 1].iter() {
        for j in [-1, 0, 1].iter() {
            let cost = if *i == 0 || *j == 0 {UNIT_DIST} else {DIAG_DIST};
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
                8 => vec.push(((x, y, z - 1), UNIT_DIST)),
                9 => vec.push(((x, y, z + 1), UNIT_DIST)),
                _ => (),
            }
        }
    }

    vec
}
 
