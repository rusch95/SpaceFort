use std::path::Path;

use entities::interact::{Action, Actions, ActionType};
use entities::pathfind::path_next_to;
use entities::creatures::{CreatureID, CreatureMap, init_creatures, dig_speed};
use game::base::{EntID, GameState, PlayerState, PlayerID, TeamID};
use map::tiles::Map;

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;
pub type EntIds = Vec<EntID>;
pub type Entities = Vec<Entity>;

pub type Health = i32;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub id: EntID,
    pub creature_id: CreatureID,
    pub pos: Pos,
    pub team_id: TeamID,
    pub actions: Actions,
    pub goal: Option<ActionType>,
    pub health: Health,
}


impl Entity {
    fn new(id: EntID, pos: Pos, team_id: PlayerID) -> Entity {
        Entity { 
            id: id, 
            creature_id: 1,
            pos: pos, 
            team_id: Some(team_id),
            actions: Actions::new(), 
            goal: None,
            health: 100,
        }
    }
}


pub fn init_entities(root: &Path) -> (Entities, CreatureMap) {

    let creature_types = init_creatures(root);
    let mut ents = Entities::new();

    let entity1 = Entity::new(-1, (7, 7, 0), 1);
    let entity2 = Entity::new(-2, (3, 3, 0), 1);
    let entity3 = Entity::new(-3, (4, 4, 0), 1);
    let entity4 = Entity::new(-4, (80, 7, 0), 2);
    let entity5 = Entity::new(-5, (83, 3, 0), 2);
    let entity6 = Entity::new(-6, (85, 4, 0), 2);

    ents.push(entity1);
    ents.push(entity2);
    ents.push(entity3);
    ents.push(entity4);
    ents.push(entity5);
    ents.push(entity6);
        
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
