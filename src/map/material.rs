use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use toml;

pub type MaterialID = u16;
pub type Materials = HashMap<MaterialID, Material>;
type ProtoMaterials = Vec<ProtoMaterial>;
type ProtoMap = HashMap<String, ProtoMaterial>;

#[derive(Deserialize)]
struct DesMaterials {
    pub materials: ProtoMaterials
}

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    // The Stuff dreams are made of
    pub name: String,
    pub id: MaterialID,
    pub texture: Option<String>,
    pub diggable: bool,
    pub passable: bool,
    pub color: [f32; 4],
    pub alt: MaterialID,
}


#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProtoMaterial {
    // The Stuff dreams are made of
    pub name: String,
    pub id: MaterialID,
    pub template: Option<String>,
    pub texture: Option<String>,
    pub diggable: Option<bool>,
    pub passable: Option<bool>,
    pub color: Option<[f32; 4]>,
    pub alt: Option<MaterialID>,
}

// TODO Genercize and dedup object, entity, and material 
pub fn init_materials(root: &Path) -> Materials {
    // Read materials file to str
    let material_path = root.join("static/inc/materials/materials.toml");
    let path_str = material_path
                   .to_str()
                   .unwrap();
    let mut file = File::open(&path_str).unwrap();
    let mut contents = String::new(); 
    file.read_to_string(&mut contents).unwrap();

    let des_materials: DesMaterials = toml::from_str(&contents)
                                            .expect("materials.toml invalid");
    let proto_materials = des_materials.materials.clone();
            
    let mut proto_map = HashMap::new();
    for mat in &proto_materials {
        proto_map.insert(mat.name.clone(), mat.clone());
    }

    let mut material_map = HashMap::new();
    // Alternatively, one could topologically sort based on dependencies
    // No current checking for circular dependencies
    for mat in &proto_materials {
        resolve(mat, &proto_map, &mut material_map);
    }

    material_map
}

// TODO Kill these unnecessary clones
fn resolve(proto: &ProtoMaterial, proto_map: &ProtoMap, mut material_map: &mut Materials) {
    match proto.template.clone() {
        None => {
            let new_mat = Material { name:     proto.name.clone(),
                                     id:       proto.id,
                                     texture:   None,  // FIXME
                                     diggable: proto.diggable.unwrap(),
                                     passable: proto.passable.unwrap(),
                                     alt:      proto.alt.unwrap(),
                                     color:    proto.color.unwrap() };
            material_map.insert(new_mat.id, new_mat);
        },
        Some(template) => {
            let template_proto = proto_map.get(&template)
                                          .unwrap();
            if !material_map.contains_key(&template_proto.id) {
                resolve(template_proto, proto_map, &mut material_map);
            }

            // Initialize with prototype and then overwrite fields with any new field
            let mut new_mat = material_map.get(&template_proto.id)
                                          .unwrap()
                                          .clone();
            new_mat.name = proto.name.clone();
            new_mat.id = proto.id;
            if let Some(texture) = proto.texture.clone() {
                new_mat.texture = Some(texture);
            }
            if let Some(diggable) = proto.diggable {
                new_mat.diggable = diggable;
            }
            if let Some(passable) = proto.passable {
                new_mat.passable = passable;
            }
            if let Some(color) = proto.color {
                new_mat.color = color;
            }
            if let Some(alt) = proto.alt {
                new_mat.alt = alt;
            }
            material_map.insert(new_mat.id, new_mat);
        }
    }
}
