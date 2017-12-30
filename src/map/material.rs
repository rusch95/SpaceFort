use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::collections::HashMap;
use toml;


pub type MaterialID = u16;
pub type Materials = HashMap<MaterialID, Material>;
type ProtoMap = HashMap<String, ProtoMaterial>;

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub name: String,
    pub id: MaterialID,
    pub texture: Option<String>,
    pub diggable: bool,
    pub passable: bool,
    pub color: [f32; 4],
    pub alt: MaterialID,
}

#[derive(Deserialize)]
struct DesMat {
    pub materials: Vec<ProtoMaterial>,
}

#[derive(Clone, PartialEq, Deserialize)]
struct ProtoMaterial {
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
    let path_str = material_path.to_str().unwrap();
    let mut file = File::open(&path_str).unwrap();
    let mut contents = String::new(); 
    file.read_to_string(&mut contents).unwrap();

    // Deserialize
    let des_mat: DesMat = toml::from_str(&contents).unwrap();
    let proto_materials = des_mat.materials;
    let proto_map: ProtoMap = proto_materials.iter()
                                             .map(|mat| (mat.name.clone(), mat.clone()))
                                             .collect();

    let mut material_map = HashMap::new();
    for mat in &proto_materials {
        resolve(mat, &proto_map, &mut material_map)
    }
    material_map
}

fn resolve(proto: &ProtoMaterial, proto_map: &ProtoMap, mut material_map: &mut Materials) {
    let mat = match proto.template {
        None => {
            Material { 
                name:     proto.name.clone(),
                id:       proto.id,
                texture:   None,  // FIXME
                diggable: proto.diggable.unwrap(),
                passable: proto.passable.unwrap(),
                alt:      proto.alt.unwrap(),
                color:    proto.color.unwrap() 
            }
        },
        Some(ref template) => {
            let template_proto = proto_map.get(template).unwrap();
            if !material_map.contains_key(&template_proto.id) {
                resolve(template_proto, proto_map, &mut material_map);
            }

            // Initialize with prototype 
            let mut mat = material_map.get(&template_proto.id).unwrap().clone();

            // Overwrite applicable fields
            mat.name = proto.name.clone();
            mat.id = proto.id;
            if let Some(texture) = proto.texture.clone() {
                mat.texture = Some(texture);
            }
            if let Some(diggable) = proto.diggable {
                mat.diggable = diggable;
            }
            if let Some(passable) = proto.passable {
                mat.passable = passable;
            }
            if let Some(color) = proto.color {
                mat.color = color;
            }
            if let Some(alt) = proto.alt {
                mat.alt = alt;
            }

            mat
        }
    };
    material_map.insert(mat.id, mat);
}
