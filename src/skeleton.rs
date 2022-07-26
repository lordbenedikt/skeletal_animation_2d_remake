use crate::*;
use bevy::sprite::MaterialMesh2dBundle;
use bone::Bone;
use cloth::Cloth;
use skin::Skin;

const VERTEX_BONE_MAX_DISTANCE: f32 = 1.;

#[derive(Default)]
pub struct Skeleton {
    pub bones: Vec<Entity>,
    pub skin_mapping: SkinMapping,
}
impl Skeleton {
    pub fn remove_bone(&mut self, bone: Entity) {
        self.bones.retain(|&b| bone != b);
        self.skin_mapping.remove_bone(bone);
    }
}

#[derive(Default)]
pub struct SkinMapping {
    pub skins: Vec<Entity>,
    pub vertex_mappings: Vec<Vec<VertexMapping>>,
}
impl SkinMapping {
    pub fn remove_vertex(&mut self) {}
    pub fn remove_bone(&mut self, bone: Entity) {
        for item in self.vertex_mappings.iter_mut() {
            for mapping in item.iter_mut() {
                for i in (0..mapping.bones.len()).rev() {
                    if mapping.bones[i] == bone {
                        mapping.bones.swap_remove(i);
                        mapping.weights.swap_remove(i);
                        mapping.rel_positions.swap_remove(i);
                        mapping.normalize();
                    }
                }
            }
        }
    }
    pub fn remove_bone_at(&mut self, index: usize) {
        for item in self.vertex_mappings.iter_mut() {
            for mapping in item.iter_mut() {
                mapping.bones.swap_remove(index);
                mapping.weights.swap_remove(index);
                mapping.rel_positions.swap_remove(index);
                mapping.normalize();
            }
        }
    }
}

#[derive(Default)]
pub struct VertexMapping {
    is_free: bool,
    weights: Vec<f32>,
    bones: Vec<Entity>,
    rel_positions: Vec<Vec2>,
}
impl VertexMapping {
    fn normalize(&mut self) {
        let mut total = 0.;
        for weight in self.weights.iter() {
            total += *weight;
        }
        for weight in self.weights.iter_mut() {
            *weight /= total;
        }
    }
    fn refine(&mut self) {
        let mut min = 9999999.;
        for weight in self.weights.iter() {
            if *weight < min {
                min = *weight;
            }
        }
        for i in (0..self.weights.len()).rev() {
            if self.weights[i] > VERTEX_BONE_MAX_DISTANCE && self.weights[i] > min {
                self.weights.swap_remove(i);
                self.rel_positions.swap_remove(i);
                self.bones.swap_remove(i);
            }
        }
        for weight in self.weights.iter_mut() {
            *weight = f32::powi(VERTEX_BONE_MAX_DISTANCE - *weight, 10);
        }
    }
    fn clear(&mut self) {
        self.weights.clear();
        self.bones.clear();
        self.rel_positions.clear();
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(assign_skins_to_bones)
        .with_system(apply_mesh_to_skeleton)
}

fn add_skin(
    mut commands: &mut Commands,
    mut meshes: &mut Assets<Mesh>,
    mut materials: &mut Assets<ColorMaterial>,
    mut skeleton: &mut skeleton::Skeleton,
    asset_server: &AssetServer,
    filename: &str,
    cols: u16,
    rows: u16,
    depth: f32,
) -> (Entity, Mesh2dHandle) {
    let mut skin = Skin::grid_mesh(filename, cols, rows, depth);

    let vertices = skin
        .vertices
        .clone()
        .iter()
        .map(|v| [v[0], v[1], depth])
        .collect::<Vec<[f32; 3]>>();
    let mut normals = vec![];
    let uvs = skin.uvs.clone();
    for _ in skin.vertices.iter() {
        normals.push([0., 0., 1.]);
    }
    let mut inds = skin.indices.clone();
    inds.reverse();
    let indices = Some(Indices::U16(inds));

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
    mesh.set_indices(indices.clone());

    let handle: Mesh2dHandle = meshes.add(mesh).into();
    skin.mesh_handle = Some(handle.clone());

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: handle.clone(),
        material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
        ..default()
    });
    let skin_id = commands
        .spawn_bundle(TransformBundle::from_transform(Transform {
            scale: Vec3::new(3.5, 3.5, 1.),
            ..default()
        }))
        .insert(Transformable {
            is_selected: false,
            ..default()
        })
        .insert(skin)
        .id();
    skeleton.skin_mapping.skins.push(skin_id);
    (skin_id, handle.clone())
}

pub fn add_skins(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut skeleton: ResMut<skeleton::Skeleton>,
    asset_server: Res<AssetServer>,
) {
    // add_skin(
    //     &mut commands,
    //     &mut meshes,
    //     &mut materials,
    //     &mut skeleton,
    //     &asset_server,
    //     "person.png",
    //     40,
    //     40,
    //     100.,
    // );
    // let (entity, mesh_handle) = add_skin(
    //     &mut commands,
    //     &mut meshes,
    //     &mut materials,
    //     &mut skeleton,
    //     &asset_server,
    //     "loechrig.png",
    //     20,
    //     20,
    //     90.,
    // );
    // let cloth = Cloth::from_mesh(mesh_handle, &meshes);
    // commands.entity(entity).insert(cloth);

    // let (entity, _) = add_skin(
    //     &mut commands,
    //     &mut meshes,
    //     &mut materials,
    //     &mut skeleton,
    //     &asset_server,
    //     "test_cloth.png",
    //     10,
    //     10,
    //     90.,
    // );
    // let cloth = Cloth::new(Vec3::new(0., 0., 0.), 5., 4., 10, 10);
    // commands.entity(entity).insert(cloth);

    // // Add Cobra Snake
    // let (entity, mesh_handle) = add_skin(
    //     &mut commands,
    //     &mut meshes,
    //     &mut materials,
    //     &mut skeleton,
    //     &asset_server,
    //     "cobra.png",
    //     25,
    //     20,
    //     90.,
    // );

    // Add Spinnenmann
    let (entity, mesh_handle) = add_skin(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut skeleton,
        &asset_server,
        "img/pooh.png",
        30,
        30,
        90.,
    );

    let (entity, _) = add_skin(
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut skeleton,
        &asset_server,
        "spider_web.png",
        10,
        10,
        90.,
    );
    let cloth = Cloth::new(Vec3::new(0., 0., 0.), 5., 4., 10, 10).with_stiffness(10);
    commands.entity(entity).insert(cloth);
}

pub fn assign_skins_to_bones(
    keys: Res<Input<KeyCode>>,
    mut skeleton: ResMut<Skeleton>,
    q0: Query<(&GlobalTransform, &Skin, &Transformable, Option<&Cloth>)>,
    q1: Query<&GlobalTransform, With<Bone>>,
) {
    // assign skins to bones when A is pressed
    if !keys.just_pressed(KeyCode::A) {
        return;
    }

    skeleton.skin_mapping.vertex_mappings.clear();

    // For each SKIN
    for skin_index in 0..skeleton.skin_mapping.skins.len() {
        skeleton.skin_mapping.vertex_mappings.push(vec![]);
        match q0.get(skeleton.skin_mapping.skins[skin_index]) {
            Ok(result) => {
                // let transformable = result.2;
                // if !transformable.is_selected {
                //     continue;
                // }

                let skin = result.1; // get skin
                let opt_cloth = result.3;
                let gl_transform = result.0; // get skin global transform
                let skin_vertices = skin.gl_vertices(gl_transform); // get vertex global position

                // Add a WEIGHTING for each vertex
                for i in 0..skin.vertices.len() {
                    // create a weighting for each vertex
                    let mut mapping = VertexMapping::default();

                    if opt_cloth.is_some() && opt_cloth.unwrap().vertex_is_free(i) {
                        // If vertex is free no need to calculate weighting
                        mapping.is_free = true;
                    } else {
                        // Assign a weight for each bone
                        for bone_index in 0..skeleton.bones.len() {
                            match q1.get(skeleton.bones[bone_index]) {
                                Ok(bone_gl_transform) => {
                                    // Calculate distance from vertex to bone
                                    let v = Vec2::from_slice(&skin_vertices[i]);
                                    let start = bone_gl_transform.translation.truncate();
                                    let end = Bone::get_tip(bone_gl_transform);
                                    let distance = transform::distance_segment_point(start, end, v);
                                    // let distance_scaled = distance / bone_gl_transform.scale.y;

                                    // Calculate vertex position relative to bone
                                    let mut rel_position = Vec3::from_slice(&skin_vertices[i]);
                                    rel_position -= bone_gl_transform.translation;
                                    rel_position = Quat::mul_vec3(
                                        bone_gl_transform.rotation.inverse(),
                                        rel_position,
                                    );
                                    if bone_gl_transform.scale.x != 0.
                                        && bone_gl_transform.scale.y != 0.
                                        && bone_gl_transform.scale.z != 0.
                                    {
                                        rel_position /= bone_gl_transform.scale;
                                    } else {
                                        println!("assign_skin_to_bones: Failed to compute relative position, because origin's scale is 0");
                                    }

                                    mapping.bones.push(skeleton.bones[bone_index]);
                                    mapping.weights.push(distance);
                                    mapping.rel_positions.push(rel_position.truncate());
                                }
                                Err(_) => continue,
                            };
                        }
                        mapping.refine(); // Remove bones that are too far from mapping
                        mapping.normalize(); // normalize weighting
                    }

                    skeleton.skin_mapping.vertex_mappings[skin_index].push(mapping);
                    // and push to skeleton vertex weightings
                }
            }
            Err(_) => continue,
        };
    }
}

pub fn apply_mesh_to_skeleton(
    mut meshes: ResMut<Assets<Mesh>>,
    mut skeleton: ResMut<Skeleton>,
    q_bones: Query<&GlobalTransform, With<Bone>>,
    q_skins: Query<&Skin>,
) {
    let skin_mapping = &mut skeleton.skin_mapping;
    if skin_mapping.vertex_mappings.is_empty() {
        return;
    }

    // for each SKIN
    for i in 0..skin_mapping.skins.len() {
        let mut vertices: Vec<[f32; 3]> = vec![];

        // if skin doesn't exist, continue
        let opt_skin = q_skins.get(skin_mapping.skins[i]);
        if opt_skin.is_err() {
            continue;
        }
        let skin = opt_skin.unwrap();

        // if mesh doesn't exist, continue
        let opt_mesh = meshes.get_mut(skin.mesh_handle.clone().unwrap().0);
        if opt_mesh.is_none() {
            continue;
        }
        let mesh = opt_mesh.unwrap();

        // for each VERTEX
        for v_i in 0..skin.vertices.len() {
            // if vertex is free keep old position and continue to next vertex
            if skin_mapping.vertex_mappings[i][v_i].is_free == true {
                vertices.push(mesh::get_vertex(mesh, v_i));
                continue;
            }
            let mut v_gl_position = Vec3::new(0., 0., 0.);
            // for each BONE
            for b_i in (0..skin_mapping.vertex_mappings[i][v_i].bones.len()).rev() {
                let bone = skin_mapping.vertex_mappings[i][v_i].bones[b_i];
                if let Ok(bone_gl_transform) = q_bones.get(bone) {
                    let weight = skin_mapping.vertex_mappings[i][v_i].weights[b_i];
                    let mut position =
                        skin_mapping.vertex_mappings[i][v_i].rel_positions[b_i].extend(0.);
                    position = Quat::mul_vec3(bone_gl_transform.rotation, position);
                    position *= bone_gl_transform.scale;
                    position += bone_gl_transform.translation;
                    v_gl_position += weight * position;
                } else {
                    skin_mapping.remove_bone(bone);
                    continue;
                }
            }
            vertices.push([v_gl_position.x, v_gl_position.y, skin.depth]);
        }

        // update mesh vertices
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    }
}

// pub fn create_textured_mesh(
//     mut commands: Commands,
//     cursor_pos: Res<CursorPos>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     let mut skin = skin::generate_mesh("left_leg.png");
//     let mut normals = vec![];
//     let mut uvs = vec![];
//     for vertex in skin.vertices.iter() {
//         normals.push([0.,0.,1.]);
//         uvs.push([vertex[0] / skin.dimensions[0] as f32, 1. - vertex[1] / skin.dimensions[1] as f32]);
//     }
//     let mut inds = skin.indices.clone();
//     inds.reverse();
//     let indices = Some(Indices::U16(inds));

//     match skin.mesh_handle.clone() {
//         Some(mesh_handle) => {
//             let _mesh = meshes.get_mut(&mesh_handle.0).unwrap();
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skin.vertices.clone());
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
//             // _mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
//             _mesh.set_indices(indices.clone());
//         }
//         None => {
//             let mut textured_mesh = Mesh::new(PrimitiveTopology::TriangleList);
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skin.vertices.clone());
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
//             textured_mesh.set_indices(indices.clone());

//             let handle: Mesh2dHandle = meshes.add(textured_mesh).into();
//             skin.mesh_handle = Some(handle.clone());

//             commands.spawn_bundle(MaterialMesh2dBundle {
//                 mesh: handle,
//                 // transform: Transform::default().with_scale(Vec3::splat(0.005)),
//                 material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
//                 ..default()
//             });
//         }
//     }
// }
