use std::collections::VecDeque;
use std::cmp::{min, max};
use map::tiles::Map;
use entities::entity::{Entities, EntIds, Pos, Ticks};
use io::tiles::{TilesSelector, Id};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Action {
    pub atype: ActionType,
    pub duration: Ticks
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionType {
    Move(Pos),
    Dig(Pos),
    Attack(Id),
    Wait,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Task {
    pub atype: ActionType,
    pub owner: Option<Id>,
}

impl Task {
    fn dig(pos: Pos) -> Task {
        Task { atype: ActionType::Dig(pos), owner: None }
    }
}

pub type Actions = VecDeque<Action>;
pub type Tasks = Vec<Task>;

pub fn select_entities(ents: &Entities, selector: TilesSelector) -> EntIds {
    let (s1, s2) = rotate_selector(selector);

    let mut ent_ids = EntIds::new();
    for ent in ents {
        if s1 <= ent.pos && ent.pos <= s2 {
            ent_ids.push(ent.id);
        }
    }
    ent_ids
}

pub fn add_dig_tasks(tasks: &mut Tasks, map: &mut Map, selector: TilesSelector) {
    let ((x1, y1, z1), (x2, y2, z2)) = rotate_selector(selector);

    // TODO Fix double dig
    for x in x1..(x2 + 1) {
        for y in y1..(y2 + 1) {
            for z in z1..(z2 + 1) {
                if let Some(tile) = map.get_tile((x, y, z)) {
                    if tile.material == 1 && tile.marked == false {
                        map.mark((x, y, z));
                        tasks.push(Task::dig((x, y, z)));
                    }
                }
            }
        }
    }
}

// Make top left corner first element and bottom left corner second element
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
