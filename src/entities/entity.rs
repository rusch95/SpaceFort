use std::path::Path;

use entities::interact::{Action, Actions, ActionType};
use entities::pathfind::path_next_to;
use entities::creatures::{CreatureID, CreatureMap, init_creatures, dig_speed};
use game::base::{EntID, GameState, PlayerState, TeamID};
use map::tiles::Map;

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;
pub type EntIds = Vec<EntID>;
pub type Entities = Vec<Entity>;


#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub id: EntID,
    pub creature_id: CreatureID,
    pub pos: Pos,
    pub team_id: TeamID,
    pub actions: Actions,
    pub goal: Option<ActionType>,
}


impl Entity {
    fn new(id: EntID, pos: Pos) -> Entity {
        Entity { 
            id: id, 
            creature_id: 1,
            pos: pos, 
            team_id: None,
            actions: Actions::new(), 
            goal: None 
        }
    }
}


pub fn init_entities(root: &Path) -> (Entities, CreatureMap) {

    let creature_types = init_creatures(root);
    let mut ents = Entities::new();

    let entity = Entity::new(-1, (7, 7, 0));
    let entity2 = Entity::new(-2, (3, 3, 0));
    let entity3 = Entity::new(-3, (4, 4, 0));

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


pub fn schedule_actions(g_state: &mut GameState, p_state: &mut PlayerState) {
    for ent in &mut g_state.entities {
        if ent.actions.is_empty() {
            for task in &mut p_state.tasks {
                if task.owner == None {
                    task.owner = Some(ent.id);
                    ent.actions = schedule_action(&g_state.map, ent, &g_state.creature_types, 
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
