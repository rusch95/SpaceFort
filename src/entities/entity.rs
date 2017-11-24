use std::path::Path;

use entities::interact::{Action, Actions, ActionType};
use entities::pathfind::path_next_to;
use entities::creatures::{CreatureID, CreatureMap, init_creatures, dig_speed};
use io::base::Id;
use game::base::GameState;
use map::tiles::Map;

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;
pub type EntIds = Vec<Id>;
pub type Entities = Vec<Entity>;


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
}


pub fn init_entities(root: &Path) -> (Entities, CreatureMap) {

    let creature_types = init_creatures(root);
    let mut ents = Entities::new();

    let entity = Entity::new(-1, (0, 0, 1));
    let entity2 = Entity::new(-2, (3, 3, 1));
    let entity3 = Entity::new(-3, (4, 4, 1));

    ents.push(entity);
    ents.push(entity2);
    ents.push(entity3);
        
    (ents, creature_types)
}


pub fn do_actions(state: &mut GameState) {
    for ent in &mut state.entities {
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
    for ent in &mut state.entities {
        if ent.actions.is_empty() {
            for task in &mut state.tasks {
                if task.owner == None {
                    task.owner = Some(ent.id);
                    ent.actions = schedule_action(&state.map, ent, &state.creature_types, 
                                                  task.atype);
                    break;
                }
            }
        }
    }
}


fn schedule_action(map: &Map, ent: &Entity, creature_types: &CreatureMap, 
                   atype: ActionType) -> Actions {
    let mut actions = Actions::new();
    match atype {
        ActionType::Dig(pos) => {
            let path = path_next_to(map, ent, creature_types, pos);
            if !path.is_empty() {
                actions.extend(path_next_to(map, ent, creature_types, pos));
                actions.push_back(Action{ atype: ActionType::Dig(pos), 
                                          duration: dig_speed(&ent.creature_id, creature_types) });
            }
        }
        _ => (),
    }

    actions
}
