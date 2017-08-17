use std::collections::VecDeque;
use std::cmp::{min, max};
use entities::entity::{Entities, EntIds, Pos, Ticks};
use io::tiles::{TilesSelector};

pub type Actions = VecDeque<Action>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Action {
    pub atype: ActionType,
    pub duration: Ticks
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActionType {
    Move(Pos),
    Wait
}

pub fn select_entities(ents: &Entities, selector: TilesSelector) -> EntIds {
    let (s1, s2) = rotate_selector(selector);

    let mut ent_ids = EntIds::new();
    for ent in ents {
        if s1 <= ent.pos && ent.pos <= s2 {
            ent_ids.push(ent.id);
        }
    }
    println!("{:?}", ent_ids);
    ent_ids
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
