use entities::actions::{Action, Goal};
use entities::creatures::CreatureMap;
use entities::entity::{Entities, EntID};
use entities::entity::{do_actions, resolve_dead};
use entities::pathfind::{path_to, path_next_to};
use map::tiles::{Map, PosUnit};


pub const FRAME_RATE_NS: u32 = 16_666_667;
const VALIDATION_PERIOD: i32 = 10;

pub type PlayerID = u16;
pub type TeamID = Option<PlayerID>;
pub type Pos = (PosUnit, PosUnit, PosUnit);
pub type Ticks = i32;

pub enum Change {
    TileChange(Pos),
    EntChange(EntID),
}

pub struct GameState {
    pub map: Map,
    pub creature_types: CreatureMap,
    pub entities: Entities,
    pub ticks: Ticks,
    #[allow(dead_code)]
    pub cur_id: EntID, // Global state for giving things ids
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum NewGoalPoss {
    NoChange,
    Delete(EntID),
    NewGoal(EntID, Goal),
}

impl GameState {
    // Contains all state corresponding to a running game
    pub fn new(map: Map, entities: Entities, creature_types: CreatureMap) -> GameState {
        GameState {
            map: map,
            creature_types: creature_types,
            entities: entities,
            ticks: 0,
            cur_id: 0,
        }
    }

    pub fn update(&mut self) -> Vec<Change> {
        self.ticks += 1;

        // Fix or null any invalid goals
        if self.ticks % VALIDATION_PERIOD == 0 {
            self.validate_goals();
        }
        // Entity update and pathfinding
        let changes = do_actions(&mut self.entities, &mut self.map);
        resolve_dead(&mut self.entities);

        changes
    }

    pub fn move_ents(&mut self,  ent_ids: &[EntID], dest_pos: Pos) {
        for ent in &mut self.entities {
            for ent_id in ent_ids {
                if ent.id == *ent_id {
                    ent.actions = path_to(&self.map, ent,
                                          &self.creature_types, dest_pos);
                }
            }
        }
    }

    pub fn validate_goals(&mut self) {
        use self::NewGoalPoss::*;

        // Generate new goals
        let new_goals: Vec<NewGoalPoss> = self.entities
            .iter()
            .filter(|ent| ent.goal.is_some())
            .map(|ent| {
                match ent.goal {
                    Some(Goal::Attack(attack_type, ent_id, pos)) => {
                        if let Some(t_ent) = self.entities.iter()
                                                          .find(|t_ent| t_ent.id == ent_id) {
                            if !t_ent.alive {
                                Delete(ent.id) 
                            } else if t_ent.pos != pos || ent.actions.is_empty() {
                                let goal = Goal::Attack(attack_type, ent_id, t_ent.pos);
                                NewGoal(ent.id, goal)
                            } else {
                                NoChange
                            }
                        } else {
                            Delete(ent.id)
                        }
                    },
                    None => NoChange,
                }})
            .filter(|goal| goal != &NoChange)
            .collect();

        // Implement new goals
        for goal in new_goals.iter() {
            match *goal {
                NewGoal(ent_id, Goal::Attack(attack_type, id, pos)) => {
                    let ent = self.entities.iter_mut()
                                           .find(|ent| ent.id == ent_id)
                                           .unwrap();
                    ent.actions = path_next_to(&self.map, ent, 
                                               &self.creature_types, pos);
                    let (action, goal) = Action::attack(id, pos, ent.creature_id,
                                                        &self.creature_types);
                    ent.actions.push_back(action);
                    ent.goal = Some(goal);
                },
                Delete(ent_id) => {
                    let ent = self.entities.iter_mut()
                                           .find(|ent| ent.id == ent_id)
                                           .unwrap();
                    ent.goal = None;
                    ent.actions.clear();
                },
                _ => unimplemented!(),
            }
        }
    }

    #[allow(dead_code)]
    pub fn give_id(&mut self) -> EntID {
        self.cur_id += 1;
        self.cur_id
    }
}

