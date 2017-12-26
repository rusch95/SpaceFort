use std::collections::VecDeque;
use std::cmp::{min, max};

use map::tiles::Map;
use entities::entity::{Entity, EntID, EntIDs};
use game::base::*;
use io::base::TilesSelector;


pub type Actions = VecDeque<Action>;
pub type Tasks = Vec<Task>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Action {
    pub atype: ActionType,
    pub duration: Ticks
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionType {
    Move(Pos),
    Dig(Pos),
    Attack(EntID),
    #[allow(dead_code)]
    Wait,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
/// Actions needing to be done
pub struct Task {
    pub atype: ActionType,
    pub owner: Option<EntID>,
}

impl Action {
    // TODO Unhardcode attack duration
    pub fn attack(ent_id: EntID) -> Action {
        Action { atype: ActionType::Attack(ent_id), duration: 10 }
    }
}

impl Task {
    /// Schedule a tile to be dug
    ///
    /// # Arguments
    ///
    /// * `pos` - The position of the tile to be dug
    fn dig(pos: Pos) -> Task {
        Task { atype: ActionType::Dig(pos), owner: None }
    }
}

// TODO Refactor into having a filter Predicate supplied
pub fn select_entities<F>(pred: F, ents: &[Entity], 
                       selector: TilesSelector) -> EntIDs
    where F: Fn(&Entity) -> bool {
    let (s1, s2) = rotate_selector(selector);

    ents.iter()
        .filter(|ent| s1 <= ent.pos && 
                ent.pos <= s2 && 
                pred(ent))
        .map(|ent| ent.id)
        .collect()
}

pub fn add_dig_tasks(tasks: &mut Tasks, map: &mut Map, selector: TilesSelector) {
    let ((x1, y1, z1), (x2, y2, z2)) = rotate_selector(selector);

    for x in x1..(x2 + 1) {
        for y in y1..(y2 + 1) {
            for z in z1..(z2 + 1) {
                if map.diggable((x, y, z)) {
                    map.mark((x, y, z));
                    tasks.push(Task::dig((x, y, z)));
                }
            }
        }
    }
}

// Make top left corner first element and bottom left corner second element
/// Re-paramaterize the rectangular selection as the bottomost corner and topmost corner
fn rotate_selector(selector: TilesSelector) -> TilesSelector {
    let ((x1, y1, z1), (x2, y2, z2)) = selector;
    let nx1 = min(x1, x2);
    let nx2 = max(x1, x2);
    let ny1 = min(y1, y2);
    let ny2 = max(y1, y2);
    let nz1 = min(z1, z2);
    let nz2 = max(z1, z2);
    ((nx1, ny1, nz1), (nx2, ny2, nz2))
}
