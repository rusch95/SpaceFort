use entities::interact::{Action, Actions, ActionType};

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub pos: Pos,
    pub actions: Actions,
}

pub type Entities = Vec<Entity>;

pub fn init_entities() -> Entities {

    let mut entities = Entities::new();

    let mut actions = Actions::new();
    for i in 0..30 {
        actions.push_back(Action { atype: ActionType::Move((i, i, 0)), duration: 20 });
    }

    entities.push(
        Entity { pos: (0, 0, 0), 
                 actions: actions });
    entities
}
