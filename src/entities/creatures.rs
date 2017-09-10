use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use entities::entity::Ticks;
use toml;

pub type CreatureID = u16;
pub type Creatures = HashMap<CreatureID, Creature>;
type ProtoCreatures = Vec<ProtoCreature>;
type ProtoCreatureMap = HashMap<String, ProtoCreature>;


#[derive(Deserialize)]
struct DeserializeStruct {
    pub creatures: ProtoCreatures,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Creature {
    // The Stuff dreams are made of
    pub name: String,
    pub id: CreatureID,
    pub texture: Option<String>,
    pub dig_speed: Ticks,
    pub movement_speed: Ticks,
    pub color: [f32; 4],
    pub alt: CreatureID,
}


#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProtoCreature {
    pub name: String,
    pub id: CreatureID,
    pub template: Option<String>,
    pub texture: Option<String>,
    pub dig_speed: Option<Ticks>,
    pub movement_speed: Option<Ticks>,
    pub color: Option<[f32; 4]>,
    pub alt: Option<CreatureID>,
}

// TODO Genercize and dedup object, entity, and material 
pub fn init_creatures(root: &Path) -> Creatures {
    // Read creatures file to str
    let creature_path = root.join("static/inc/creatures/creatures.toml");
    let path_str = creature_path
                   .to_str()
                   .unwrap();
    let mut file = File::open(&path_str).unwrap();
    let mut contents = String::new(); 
    file.read_to_string(&mut contents).unwrap();

    let des_struct: DeserializeStruct = toml::from_str(&contents)
                                              .expect("creatures.toml invalid");
    let proto_creatures = des_struct.creatures.clone();
            
    let mut proto_map = HashMap::new();
    for creat in proto_creatures.iter() {
        proto_map.insert(creat.name.clone(), creat.clone());
    }

    let mut creature_map = HashMap::new();
    // Alternatively, one could topologically sort based on dependencies
    // No current checking for circular dependencies
    for creat in proto_creatures.iter() {
        resolve(creat, &proto_map, &mut creature_map);
    }

    creature_map
}

fn resolve(proto: &ProtoCreature, proto_map: &ProtoCreatureMap, 
           mut creature_map: &mut Creatures) {
    match proto.template.clone() {
        None => {
            let new_creat = Creature { 
                name:           proto.name.clone(),
                id:             proto.id.clone(),
                texture:        None,  // FIXME
                dig_speed:      proto.dig_speed.unwrap().clone(),
                movement_speed: proto.movement_speed.unwrap().clone(),
                alt:            proto.alt.unwrap().clone(),
                color:          proto.color.unwrap().clone() };
            creature_map.insert(new_creat.id, new_creat);
        },
        Some(template) => {
            let template_proto = proto_map.get(&template)
                                          .unwrap();
            if !creature_map.contains_key(&template_proto.id) {
                resolve(template_proto, &proto_map, &mut creature_map);
            }
        }   
    }
}

