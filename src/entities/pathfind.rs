use game::base::Pos;
use map::constants::*;
use map::tiles::Map;
use entities::actions::{Action, Actions, ActionType};
use entities::entity::Entity;
use entities::creatures::{CreatureMap, movement_speed};
use entities::utils::*;

use pathfinding::fringe;


const UNIT_DIST: i32 = 100;
const DIAG_DIST: i32 = (UNIT_DIST as f64 * 1.414) as i32;

// TODO Fix distances to use f64 instead of int

pub fn path_to(map: &Map, ent: &mut Entity, creature_types: &CreatureMap, 
               end_pos: Pos) -> Actions {
    path(map, ent, creature_types, end_pos, |&p| p == end_pos)
}

pub fn path_next_to(map: &Map, ent: &Entity, creature_types: &CreatureMap, 
                    end_pos: Pos) -> Actions {
    path(map, ent, creature_types, end_pos,
         |&p| {let sucs = succ(map, &end_pos);
               sucs.contains(&(p, UNIT_DIST)) || 
               sucs.contains(&(p, DIAG_DIST))})
}

pub fn path<F>(map: &Map, ent: &Entity, creature_types: &CreatureMap, 
               end_pos: Pos, end_det: F) -> Actions where 
    F: Fn(&Pos) -> bool {    
    let pathing_result = fringe(&ent.pos,
                         |&p| succ(map, &p),
                         |&p| dist(&p, &end_pos),
                         end_det);

    let mut actions = Actions::new();
    if let Some((path, _)) = pathing_result {
        for coord in path {
            actions.push_back(
                Action { atype: ActionType::Move(coord),
                         duration: movement_speed(&ent.creature_id, creature_types) }
            );
        }
    }
    
    // First movement moves to current square, so get rid of it
    actions.pop_front();

    actions
}

fn succ(map: &Map, pos: &Pos) -> Vec<(Pos, i32)> {
    let (x, y, z) = *pos;

    let mut successors = Vec::new();
    for i in &[-1, 0, 1] {
        for j in &[-1, 0, 1] {
            // TODO Add variance to step cost
            let cost = if *i == 0 || *j == 0 {UNIT_DIST} else {DIAG_DIST};
            // Shadow X and Y with adjacent coords
            let (x, y) = (x + *i, y + *j);
            if map.passable((x, y, z)) {
                successors.push(((x, y, z), cost));
            }
        }
    };

    // Up and down
    // TODO Fix this as part of the ramp/stairs refactor
    if let Some(tile) = map.get_tile((x, y, z)) {
        match tile.mode {
            Mode::UpStairs => successors.push(((x, y, z - 1), UNIT_DIST)),
            Mode::DownStairs => successors.push(((x, y, z + 1), UNIT_DIST)),
            _ => (),
        }
    }

    successors
}
