use entities::interact::{Action, Actions, ActionType};

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;
pub type EntId = i32;
pub type EntIds = Vec<EntId>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub id: EntId,
    pub pos: Pos,
    pub actions: Actions,
}

impl Entity {
    fn new(id: EntId, pos: Pos) -> Entity {
        Entity { id: id, pos: pos, actions: Actions::new() }
    }
}
        

pub type Entities = Vec<Entity>;

pub fn init_entities() -> Entities {

    let mut entities = Entities::new();

    let mut entity = Entity::new(0, (0, 0, 1));
    let entity2 = Entity::new(1, (5, 5, 1));

    entities.push(entity);
    entities.push(entity2);
        
    entities
}
