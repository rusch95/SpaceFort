use entities::interact::{Action, Actions, ActionType};
use entities::pathfind::path_next_to;
use io::tiles::{Game, Id};
use map::tiles::Map;

pub type Pos = (i32, i32, i32);
pub type Ticks = i32;
pub type EntIds = Vec<Id>;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Entity {
    pub id: Id,
    pub pos: Pos,
    pub actions: Actions,
    pub movement_speed: Ticks,
    pub goal: Option<ActionType>,
}

impl Entity {
    fn new(id: Id, pos: Pos) -> Entity {
        Entity { id: id, 
                pos: pos, 
                movement_speed: 20,
                actions: Actions::new(), 
                goal: None }
    }
}
        

pub type Entities = Vec<Entity>;

pub fn init_entities() -> Entities {

    let mut entities = Entities::new();

    let mut entity = Entity::new(-1, (0, 0, 1));
    let entity2 = Entity::new(-2, (5, 5, 1));

    entities.push(entity);
    entities.push(entity2);
        
    entities
}

pub fn do_actions(game: &mut Game) {

    for mut ent in game.entities.iter_mut() {
        let pop = match ent.actions.front_mut() {
            Some(act) => {
                if act.duration > 0 {act.duration -= 1; false}
                else {
                    match act.atype {
                        ActionType::Move(pos) => {
                            ent.pos = pos;
                        },
                        ActionType::Dig(pos) => {
                            game.map.dig(pos)
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

pub fn schedule_actions(game: &mut Game) {

    for mut ent in game.entities.iter_mut() {
        if ent.actions.len() == 0 {
            for mut task in game.tasks.iter_mut() {
                if task.owner == None {
                    task.owner = Some(ent.id);
                    ent.actions = schedule_action(&game.map, ent, task.atype);
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
            actions.extend(path_next_to(map, ent, pos));
            actions.push_back(Action{ atype: ActionType::Dig(pos), duration: 300 });
        }
        _ => (),
    }

    actions
}
