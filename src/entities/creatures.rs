//
// TODO Genercize and dedup object, entity, and material 
pub fn init_creatures(root: &Path) -> Creatures {
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
    for mat in proto_materials.iter() {
        proto_map.insert(mat.name.clone(), mat.clone());
    }

    let mut material_map = HashMap::new();
    // Alternatively, one could topologically sort based on dependencies
    // No current checking for circular dependencies
    for mat in proto_materials.iter() {
        resolve(mat, &proto_map, &mut material_map);
    }

    material_map
}

// TODO Kill these unnecessary clones
fn resolve(proto: &ProtoMaterial, proto_map: &ProtoMap, mut material_map: &mut Materials) {
    match proto.template.clone() {
        None => {
            let new_mat = Material { name:     proto.name.clone(),
                                     id:       proto.id.clone(),
                                     texture:   None,  // FIXME
                                     diggable: proto.diggable.unwrap().clone(),
                                     passable: proto.passable.unwrap().clone(),
                                     alt:      proto.alt.unwrap().clone(),
                                     color:    proto.color.unwrap().clone() };
            material_map.insert(new_mat.id, new_mat);
        },
        Some(template) => {
            let template_proto = proto_map.get(&template)
                                          .unwrap();
            if !material_map.contains_key(&template_proto.id) {
                resolve(template_proto, &proto_map, &mut material_map);
            }

