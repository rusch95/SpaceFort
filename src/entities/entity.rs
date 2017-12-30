use std::path::Path;
use std::mem;

use entities::actions::{Action, Actions, ActionType, AttackType, Goal, Tasks};
use entities::pathfind::path_next_to;
use entities::creatures::{CreatureID, CreatureMap, init_creatures};
use game::base::*;
use map::tiles::Map;


pub type EntID = i64;
pub type EntIDs = Vec<EntID>;
pub type Entities = Vec<Entity>;
pub type EntSnaps = Vec<EntSnap>;
pub type Health = i32;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    // Id unique to each entity
    pub id: EntID,
    // Id referring to the creature type for creature properties lookup
    pub creature_id: CreatureID,
    // The entity's position in space
    pub pos: Pos,
    // The Id of the team that the entity belongs to
    pub team_id: TeamID,
    // The stack of actions that the entity has to do
    pub actions: Actions,
    // The current thing the entity wants to do e.g. attack foo or build foo
    pub goal: Option<Goal>,
    // Hit points
    pub health: Health,
    // Dead or alive
    pub alive: bool,
    // Timer for waiting out the duration of an action
    pub timer: Ticks,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
// A subset of the fields of Entity which is sent to clients for display
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
            timer: 0,
        }
    }

    pub fn snap(&self) -> EntSnap {
        EntSnap {
            id: self.id,
            creature_id: self.creature_id,
            pos: self.pos,
            team_id: self.team_id,
            health: self.health,
            alive: self.alive,
        }
    }

    pub fn attack(&mut self, target_id: EntID, ents: &mut [Entity], attack_type: AttackType) {
        // TODO Big changes. Use attack attributes from the creatures
        // file instead of a hardcoded 40. Make attackers repath if the
        // target moves. Check the the attacker is adjacent to the target

        // TODO Figure out how to work this off of creature properties
        let damage = match attack_type {
            _ => 40,
        };

        if let Some(target) = ents.iter_mut()
                                  .find(|ent| ent.id == target_id) {
            if self.is_adjacent(target) {
                target.health -= damage;
            }
        }
    }

    fn is_adjacent(&self, ent: &Entity) -> bool {
        let (sx, sy, sz)  = self.pos;
        let (ex, ey, ez) = ent.pos;

        (sx - ex).abs() <= 1 && (sy - ey).abs() <= 1 && (sz - ez).abs() <= 1 
    }

    /// Enumerate the actions the a task requires, so that they can be 
    /// added to the target entities actions queue
    ///
    /// # Returns
    /// * Actions
    /// * Can do action
    pub fn schedule_action(&self, map: &Map, creature_types: &CreatureMap, 
                       atype: ActionType) -> Option<Actions> {
        let mut actions = Actions::new();
        match atype {
            ActionType::Dig(pos) => {
                let path = path_next_to(map, &self, creature_types, pos);
                if !path.is_empty() {
                    actions.extend(path);
                    actions.push_back(Action::dig(pos, &self.creature_id, creature_types));
                    Some(actions)
                } else {
                    None
                }
            }
            _ => panic!("Not covered")
        }
    }

}

pub fn init_entities(root: &Path) -> (Entities, CreatureMap) {
    info!("Initializing entities");

    let creature_types = init_creatures(root);
    let mut ents = Entities::new();

    // TODO Get rid of this static shit and have
    // entities spawn via an actual process like
    // maybe whenever a player joins
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
    // and clear their pending actions
    // TODO Turn the dead into items and drop their items
    // and any other magic

    for ent in entities {
        if ent.health < 0 || !ent.alive {
            ent.alive = false;
            ent.team_id = None;
            ent.goal = None;
            ent.actions.clear();
        }
    }
}

pub fn do_actions(entities: &mut Entities, map: &mut Map) -> Vec<Change> {
    let mut temp_vec = Actions::new();
    // Changes keeps track of what was dug and who moved,
    // such that this can be used to selectively send 
    // updates to the clients
    // TODO Poss. investigate making generating a changes this immutably
    // and then applying all of the necessary changes
    let mut changes = Vec::<Change>::new();
    let ent_len = entities.len();
    for i in 0..ent_len {
        let (front_ents, _back_ents) = entities.split_at_mut(i);
        let (_ent, back_ents) = _back_ents.split_at_mut(1);
        let ent = &mut _ent[0];
        assert_eq!(front_ents.len() + 1 + back_ents.len(), ent_len);

        // Swap ent actions out with an empty vec to make borrow checker
        // happy while we do things with the rest of the ent
        mem::swap(&mut ent.actions, &mut temp_vec);
        // Check if task is done
        let task_done = match temp_vec.front() {
            Some(act) => {
                if act.duration > ent.timer {
                    ent.timer += 1; 
                    false
                } else {
                    ent.timer = 0;

                    match act.atype {
                        ActionType::Move(pos) => {
                            // TODO Add validation testing
                            // Shouldn't move on to invalid tile
                            ent.pos = pos;
                        },
                        ActionType::Dig(pos) => {
                            map.dig(pos);
                            changes.push(Change::TileChange(pos));
                        },
                        ActionType::Attack(attack_type, ent_id) => {
                            // As the ent list is split, we have have
                            // to search both for the target
                            ent.attack(ent_id, front_ents, attack_type);
                            ent.attack(ent_id, back_ents, attack_type);
                        },
                        _ => {},
                    };

                    true
                }
            }
            None => (false),
        };
        mem::swap(&mut ent.actions, &mut temp_vec);

        // And then pop a task if it is done
        if task_done {ent.actions.pop_front();};
    };

    changes
}

pub fn schedule_actions(entities: &mut Entities, tasks: &mut Tasks, map: &Map, 
                        creature_types: &CreatureMap, team_id: TeamID) {
    for ent in entities.iter_mut().filter(|ent| ent.actions.is_empty() &&
                                            ent.team_id == team_id) {
        tasks.sort_unstable_by_key(|task| task.priority(&ent));
        for task in tasks.iter_mut()
                         .filter(|task| task.owner.is_none()) {
            // If a task is not owned by anyone, assign it to some entitiy
            if let Some(actions) = ent.schedule_action(map, creature_types, 
                                                       task.atype) {
                task.owner = Some(ent.id);
                ent.actions = actions;
                break;
            } else {
                continue;
            }
        }
    }
}
