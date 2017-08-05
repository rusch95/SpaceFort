use std::collections::VecDeque;
use entities::entity::{Pos, Ticks};

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
