use std::path::Path;
use std::mem;

use entities::interact::{Action, Actions, ActionType, Tasks};
use entities::pathfind::path_next_to;
use entities::creatures::{CreatureID, CreatureMap, init_creatures, dig_speed};
use game::base::*;
use map::tiles::Map;

pub type EntID = i64;
pub type EntIDs = Vec<EntID>;
pub type Entities = Vec<Entity>;
pub type EntSnaps = Vec<EntSnap>;
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
    pub alive: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
// Smaller Ent for sending to clients for printing
pub struct EntSnap {
    pub id: EntID,
    pub creature_id: CreatureID,
    pub pos: Pos,
    pub team_id: TeamID,
    pub health: Health,
    pub alive: bool,
}

impl Entity {
    fn new(id: EntID, creature_id: CreatureID, pos: Pos, team_id: PlayerID) -> Entity {
        Entity { 
            id: id, 
            creature_id: creature_id,
            pos: pos, 
            team_id: Some(team_id),
            actions: Actions::new(), 
            goal: None,
            health: 100,
            alive: true,
        }
    }
}

pub fn init_entities(root: &Path) -> (Entities, CreatureMap) {
    info!("Initializing entities");

    let creature_types = init_creatures(root);
    let mut ents = Entities::new();

    let entity1 = Entity::new(-1, 1, (7, 7, 0), 1);
    let entity2 = Entity::new(-2, 1, (3, 3, 0), 1);
    let entity3 = Entity::new(-3, 1, (4, 4, 0), 1);
    let entity4 = Entity::new(-4, 2, (80, 7, 0), 2);
    let entity5 = Entity::new(-5, 2, (83, 3, 0), 2);
    let entity6 = Entity::new(-6, 2, (85, 4, 0), 2);

    ents.push(entity1);
    ents.push(entity2);
    ents.push(entity3);
    ents.push(entity4);
    ents.push(entity5);
    ents.push(entity6);
        
    (ents, creature_types)
}

pub fn resolve_dead(entities: &mut Entities) {
    // For now just mark dead as not having a team
    // Will turn the dead into items in future iterations

    for ent in entities {
        if ent.alive == false {
            ent.team_id = None;
        }
    }
}

pub fn do_actions(entities: &mut Entities, map: &mut Map) {
    let ent_len = entities.len();
    let mut temp_vec = Actions::new();
    for i in 0..entities.len() {
        let (front_ents, _back_ents) = entities.split_at_mut(i);
        let (_ent, back_ents) = _back_ents.split_at_mut(1);
        let ent = &mut _ent[0];
        assert_eq!(front_ents.len() + 1 + back_ents.len(), ent_len);

        // Swap ent actions out for a null vec to make borrow checker
        // happy while we do things when ent
        mem::swap(&mut ent.actions, &mut temp_vec);
        let pop = match temp_vec.front_mut() {
            Some(act) => {
                if act.duration > 0 {act.duration -= 1; false}
                else {
                    match act.atype {
                        ActionType::Move(pos) => {
                            ent.pos = pos;
                        },
                        ActionType::Dig(pos) => {
                            map.dig(pos)
                        },
                        ActionType::Attack(ent_id) => {
                            attack(ent, ent_id, front_ents, back_ents)
                        },
                        _ => {},
                    };

                    true
                }
            }
            None => (false),
        };
        mem::swap(&mut ent.actions, &mut temp_vec);

        if pop {ent.actions.pop_front();};
    };
}


pub fn attack(attacker: &mut Entity, defender_id: EntID,
              left_ents: &mut [Entity], right_ents: &mut [Entity]) {
    if let Some(defender) = left_ents.iter_mut()
                                     .find(|ref ent| ent.id == defender_id) {
        defender.health -= 40;
        if defender.health < 0 {
            defender.alive = false;
        }
    } else if let Some(defender) = right_ents.iter_mut()
                                      .find(|ref ent| ent.id == defender_id) {
        defender.health -= 40;
        if defender.health < 0 {
            defender.alive = false;
        }
    }
}



pub fn schedule_actions(entities: &mut Entities, tasks: &mut Tasks, 
                        map: &Map, creature_types: &CreatureMap) {
    for ent in entities {
        if ent.actions.is_empty() {
            for task in tasks.iter_mut() {
                if task.owner == None {
                    task.owner = Some(ent.id);
                    ent.actions = schedule_action(map, ent, 
                                                  creature_types, 
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
