use std::path::Path;

use entities::interact::{Action, Actions, ActionType};
use entities::pathfind::path_next_to;
use entities::creatures::{CreatureID, CreatureMap, init_creatures};
use io::base::Id;
use io::tiles::GameState;
use map::tiles::Map;

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;
pub type EntIds = Vec<Id>;
pub type Entities = Vec<Entity>;


pub struct EntState {
    pub entities: Entities,
    pub creature_types: CreatureMap,
}

impl EntState {
    fn new(creature_types: CreatureMap) -> EntState {
        EntState {
            entities: Entities::new(),
            creature_types: creature_types,
        }
    }

    fn add(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
}


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub id: Id,
    pub creature_id: CreatureID,
    pub pos: Pos,
    pub actions: Actions,
    pub goal: Option<ActionType>,
}


impl Entity {
    fn new(id: Id, pos: Pos) -> Entity {
        Entity { 
            id: id, 
            creature_id: 1,
            pos: pos, 
            actions: Actions::new(), 
            goal: None 
        }
    }

    pub fn dig_speed(&self) -> Ticks {
        300
    }

    pub fn movement_speed(&self) -> Ticks {
        15
    }
}


pub fn init_entities(root: &Path) -> EntState {

    let creature_types = init_creatures(root);
    let ent_state = EntState::new(creature_types);

    let entity = Entity::new(-1, (0, 0, 1));
    let entity2 = Entity::new(-2, (3, 3, 1));
    let entity3 = Entity::new(-3, (4, 4, 1));

    ent_state.add(entity);
    ent_state.add(entity2);
    ent_state.add(entity3);
        
    ent_state
}


pub fn do_actions(state: &mut GameState) {
    for ent in state.entities.iter_mut() {
        let pop = match ent.actions.front_mut() {
            Some(act) => {
                if act.duration > 0 {act.duration -= 1; false}
                else {
                    match act.atype {
                        ActionType::Move(pos) => {
                            ent.pos = pos;
                        },
                        ActionType::Dig(pos) => {
                            state.map.dig(pos)
                        },
                        _ => {},
                    };
                    true
                }
            }
            None => (false),
        };

        if pop {ent.actions.pop_front();};
    };
}


pub fn schedule_actions(state: &mut GameState) {
    for ent in state.entities.iter_mut() {
        if ent.actions.len() == 0 {
            for task in state.tasks.iter_mut() {
                if task.owner == None {
                    task.owner = Some(ent.id);
                    ent.actions = schedule_action(&state.map, ent, task.atype);
                    break;
                }
            }
        }
    }
}


fn schedule_action(map: &Map, ent: &Entity, atype: ActionType) -> Actions {
    let mut actions = Actions::new();
    match atype {
        ActionType::Dig(pos) => {
            let path = path_next_to(map, ent, pos);
            if path.len() > 0 {
                actions.extend(path_next_to(map, ent, pos));
                actions.push_back(Action{ atype: ActionType::Dig(pos), 
                                          duration: ent.dig_speed() });
            }
        }
        _ => (),
    }

    actions
}
