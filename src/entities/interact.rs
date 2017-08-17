use std::collections::VecDeque;
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
    let (s1, s2) = selector;

    let mut ent_ids = EntIds::new();
    for ent in ents {
        if s1 <= ent.pos && ent.pos <= s2 {
            ent_ids.push(ent.id);
        }
    }
    ent_ids
}
