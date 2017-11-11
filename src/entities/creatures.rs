use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;

use entities::entity::Ticks;
use toml;

pub type CreatureID = u16;
pub type CreatureMap = HashMap<CreatureID, Creature>;
type ProtoCreatures = Vec<ProtoCreature>;
type ProtoCreatureMap = HashMap<String, ProtoCreature>;


#[derive(Deserialize)]
struct DeserializeStruct {
    pub creatures: ProtoCreatures,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Creature {
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
pub fn init_creatures(root: &Path) -> CreatureMap {
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
           mut creature_map: &mut CreatureMap) {
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

            let mut new_creat = creature_map.get(&template_proto.id)
                                            .unwrap()
                                            .clone();
            new_creat.name = proto.name.clone();
            new_creat.id = proto.id.clone();
            if let Some(texture) = proto.texture.clone() {
                new_creat.texture = Some(texture);
            }
            if let Some(dig_speed) = proto.dig_speed.clone() {
                new_creat.dig_speed = dig_speed;
            }
            if let Some(movement_speed) = proto.movement_speed.clone() {
                new_creat.movement_speed = movement_speed;
            }
            if let Some(alt) = proto.alt.clone() {
                new_creat.alt = alt;
            }
            if let Some(color) = proto.color.clone() {
                new_creat.color = color;
            }

            creature_map.insert(new_creat.id, new_creat);
        }   
    }
}

