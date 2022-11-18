use std::{cmp, f32::consts::E};

use crate::{skin::START_SCALE, *};
use bevy::{input::mouse::{MouseWheel, MouseScrollUnit}, math::Vec3A, sprite::MaterialMesh2dBundle};
use bone::Bone;
use cloth::Cloth;
use serde::*;
use skin::Skin;

pub const VERTEX_BONE_MAX_DISTANCE: f32 = 1.;

#[derive(Resource, Default)]
pub struct Skeleton {
    pub bones: Vec<Entity>,
    pub skin_mappings: Vec<SkinMapping>,
}
impl Skeleton {
    pub fn remove_bone(&mut self, bone: Entity) {
        self.bones.retain(|&b| bone != b);
        self.skin_mappings
            .iter_mut()
            .for_each(|it| it.remove_bone(bone));
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct SkinMapping {
    pub skin: Option<Entity>,
    pub vertex_mappings: Vec<VertexMapping>,
}
impl SkinMapping {
    pub fn remove_vertex(&mut self) {}
    pub fn remove_bone(&mut self, bone: Entity) {
        for mapping in self.vertex_mappings.iter_mut() {
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
    pub fn remove_bone_at(&mut self, index: usize) {
        for mapping in self.vertex_mappings.iter_mut() {
            mapping.bones.swap_remove(index);
            mapping.weights.swap_remove(index);
            mapping.rel_positions.swap_remove(index);
            mapping.normalize();
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct VertexMapping {
    pub is_free: bool,
    pub weights: Vec<f32>,
    pub bones: Vec<Entity>,
    pub rel_positions: Vec<Vec2>,
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
    }
    fn invert(&mut self) {
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
        .with_system(apply_mesh_to_skeleton)
        .with_system(free_skins)
        .with_system(assign_skins_to_bones)
        .with_system(adjust_vertex_weights)
}

pub fn free_skins(
    keys: Res<Input<KeyCode>>,
    mut skeleton: ResMut<Skeleton>,
    mut q: Query<(&Transformable, &mut Transform), With<Skin>>,
) {
    // assign skins to bones when A is pressed
    if !(keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::A)) {
        return;
    }

    // remove skin from skeleton
    for i in (0..skeleton.skin_mappings.len()).rev() {
        if skeleton.skin_mappings[i].skin.is_none() {
            skeleton.skin_mappings.swap_remove(i);
            continue;
        }
        if let Ok((transformable, mut transform)) =
            q.get_mut(skeleton.skin_mappings[i].skin.unwrap())
        {
            if transformable.is_selected {
                skeleton.skin_mappings.swap_remove(i);
                transform.translation = Vec3::new(0., 0., 0.);
                transform.rotation = Quat::IDENTITY;
                transform.scale = Vec3::new(START_SCALE, START_SCALE, START_SCALE);
            }
        }
    }
}

pub fn assign_skins_to_bones(
    keys: Res<Input<KeyCode>>,
    mut skeleton: ResMut<Skeleton>,
    q0: Query<
        (
            &GlobalTransform,
            &Skin,
            &Transformable,
            Option<&Cloth>,
            Entity,
        ),
        Without<Bone>,
    >,
    q1: Query<(&GlobalTransform, &Transformable), With<Bone>>,
) {
    // assign skins to bones when A is pressed
    if !(!keys.pressed(KeyCode::LControl) && keys.just_pressed(KeyCode::A)) {
        return;
    }

    // Of selected skins, add new ones to skeleton and keep store indices of all skin mapping that need to be updated
    let mut relevant_skins: Vec<usize> = vec![];
    for (_, _, transformable, _, entity) in q0.iter() {
        if !transformable.is_selected {
            continue;
        }
        let mut new_skin = true;
        for i in (0..skeleton.skin_mappings.len()).rev() {
            if skeleton.skin_mappings[i].skin.is_none() {
                skeleton.skin_mappings.swap_remove(i);
                continue;
            }
            if skeleton.skin_mappings[i].skin.unwrap() == entity {
                new_skin = false;
                relevant_skins.push(i);
            }
        }
        if new_skin {
            skeleton.skin_mappings.push(SkinMapping {
                skin: Some(entity),
                vertex_mappings: vec![],
            });
            relevant_skins.push(skeleton.skin_mappings.len() - 1);
        }
    }

    // For each SKIN
    for skin_index in relevant_skins {
        match q0.get(skeleton.skin_mappings[skin_index].skin.unwrap()) {
            Ok((gl_transform, skin, skin_transformable, opt_cloth, _)) => {
                if !skin_transformable.is_selected {
                    continue;
                }

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
                                Ok((bone_gl_transform, bone_transformable)) => {
                                    // Only consider selected bones
                                    if !bone_transformable.is_selected {
                                        continue;
                                    }

                                    // Calculate distance from vertex to bone
                                    let v = Vec2::from_slice(&skin_vertices[i]);
                                    let start = bone_gl_transform.affine().translation.truncate();
                                    let end = Bone::get_tip(bone_gl_transform);
                                    let distance = transform::distance_segment_point(start, end, v);
                                    // let distance_scaled = distance / bone_gl_transform.scale.y;

                                    // Calculate vertex position relative to bone
                                    let (bone_gl_scale, bone_gl_rotation, bone_gl_translation) =
                                        bone_gl_transform.to_scale_rotation_translation();
                                    let mut rel_position = Vec3::from_slice(&skin_vertices[i]);
                                    rel_position -= bone_gl_translation;
                                    rel_position =
                                        Quat::mul_vec3(bone_gl_rotation.inverse(), rel_position);
                                    if bone_gl_scale.x != 0.
                                        && bone_gl_scale.y != 0.
                                        && bone_gl_scale.z != 0.
                                    {
                                        rel_position /= bone_gl_scale;
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
                        mapping.invert();
                        mapping.normalize(); // normalize weighting
                    }

                    skeleton.skin_mappings[skin_index]
                        .vertex_mappings
                        .push(mapping);
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
    q_bones: Query<(&Transform, Option<&Parent>), With<Bone>>,
    q_skins: Query<&Skin>,
) {
    if skeleton.skin_mappings.is_empty() {
        return;
    }

    // for each SKIN
    for i in (0..skeleton.skin_mappings.len()).rev() {
        // If vertices haven't been mapped for this skin
        if skeleton.skin_mappings[i].vertex_mappings.is_empty() {
            continue;
        }
        let mut vertices: Vec<[f32; 3]> = vec![];

        // if skin doesn't exist, continue
        if skeleton.skin_mappings[i].skin.is_none() {
            skeleton.skin_mappings.swap_remove(i);
            continue;
        }
        let opt_skin = q_skins.get(skeleton.skin_mappings[i].skin.unwrap());
        if opt_skin.is_err() {
            continue;
        }
        let skin = opt_skin.unwrap();

        // if mesh doesn't exist, continue
        let opt_mesh = meshes.get_mut(&skin.mesh_handle.clone().unwrap().0);
        if opt_mesh.is_none() {
            continue;
        }
        let mesh = opt_mesh.unwrap();

        // for each VERTEX
        for v_i in 0..skin.vertices.len() {
            // CONFUSION!!! TODO: Fix! After removing bone confusion!!
            if i >= skeleton.skin_mappings.len() {
                vertices.push(mesh::get_vertex(mesh, v_i));
                continue;
            }
            // if vertex is free keep old position and continue to next vertex
            if skeleton.skin_mappings[i].vertex_mappings[v_i].is_free == true {
                vertices.push(mesh::get_vertex(mesh, v_i));
                continue;
            }
            let mut v_gl_position = Vec3::new(0., 0., 0.);
            // for each BONE
            'outer: for b_i in (0..skeleton.skin_mappings[i].vertex_mappings[v_i].bones.len()).rev()
            {
                let bone = skeleton.skin_mappings[i].vertex_mappings[v_i].bones[b_i];
                let opt_bone_gl_transform = bone::get_bone_gl_transform(bone, &q_bones);
                let bone_gl_transform = if opt_bone_gl_transform.is_some() {
                    opt_bone_gl_transform.unwrap()
                } else {
                    skeleton.remove_bone(bone);
                    continue 'outer;
                };

                // Calculate total of all weights
                let mut total_weight: f32 = 0.0;
                skeleton.skin_mappings[i].vertex_mappings[v_i]
                    .weights
                    .iter()
                    .for_each(|&weight| total_weight += weight);

                let weight = skeleton.skin_mappings[i].vertex_mappings[v_i].weights[b_i];
                if weight == 0.0 {
                    skeleton.skin_mappings[i].vertex_mappings[v_i]
                        .bones
                        .swap_remove(b_i);
                    skeleton.skin_mappings[i].vertex_mappings[v_i]
                        .weights
                        .swap_remove(b_i);
                }

                let translation =
                    skeleton.skin_mappings[i].vertex_mappings[v_i].rel_positions[b_i].extend(0.);
                let vertex_rel_transform = Transform::from_translation(translation);
                let vertex_gl_transform =
                    combined_transform(&bone_gl_transform, &vertex_rel_transform);

                v_gl_position += weight / total_weight * vertex_gl_transform.translation;
            }

            if skeleton.skin_mappings[i].vertex_mappings[v_i].bones.len() == 0 {
                v_gl_position = Vec3::from_slice(
                    &q_skins
                        .get(skeleton.skin_mappings[i].skin.unwrap())
                        .unwrap()
                        .vertices[v_i],
                );
            }
            vertices.push([v_gl_position.x, v_gl_position.y, 0.]);
        }

        // update mesh vertices
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    }
}

fn adjust_vertex_weights(
    meshes: Res<Assets<Mesh>>,
    q: Query<(&skin::Skin, Entity)>,
    mut skeleton: ResMut<skeleton::Skeleton>,
    transform_state: Res<transform::State>,
    mut egui_state: ResMut<egui::State>,
    cursor_pos: Res<CursorPos>,
    keys: Res<Input<KeyCode>>,
    mut mouse_wheel_evr: EventReader<MouseWheel>,
) {
    if keys.just_pressed(KeyCode::W) {
        egui_state.adjust_vertex_weights_mode = !egui_state.adjust_vertex_weights_mode;
    }

    for mouse_wheel in mouse_wheel_evr.iter() {
        let change_value = mouse_wheel.y * {if mouse_wheel.unit == MouseScrollUnit::Line {0.05} else {0.0005}};
        egui_state.brush_size = f32::max(0.05, egui_state.brush_size + change_value);
    }

    let increase = if keys.just_pressed(KeyCode::E) {
        true
    } else if keys.just_pressed(KeyCode::Q) {
        false
    } else {
        return;
    };

    for (skin, entity) in q.iter() {
        let opt_mesh = meshes.get(&skin.mesh_handle.clone().unwrap().0);
        if opt_mesh.is_none() {
            continue;
        }
        let mesh = opt_mesh.unwrap();

        let vertices: Vec<Vec3> = mesh::get_vertices(mesh);

        let mut skin_mapping_index: Option<usize> = None;
        for i in 0..skeleton.skin_mappings.len() {
            if let Some(skin_entity) = skeleton.skin_mappings[i].skin {
                if skin_entity == entity {
                    skin_mapping_index = Some(i);
                }
            }
        }

        // Adjust weight for each vertex
        for i in 0..vertices.len() {
            if skin_mapping_index.is_some() && egui_state.adjust_vertex_weights_mode {
                if transform_state.selected_entities.len() > 0 {
                    // Currently selected bone
                    let bone_entity = *transform_state.selected_entities.iter().next().unwrap();
                    let v_mapping =
                        &mut skeleton.skin_mappings[skin_mapping_index.unwrap()].vertex_mappings[i];

                    // Weight of current bone for current vertex
                    for j in 0..v_mapping.bones.len() {
                        if v_mapping.bones[j] == bone_entity {
                            if vertices[i].truncate().distance(cursor_pos.0) < egui_state.brush_size
                            {
                                let value_change = 0.1;
                                if increase {
                                    v_mapping.weights[j] =
                                        f32::min(1.0, v_mapping.weights[j] + value_change);
                                } else {
                                    v_mapping.weights[j] =
                                        f32::max(0.0001, v_mapping.weights[j] - value_change);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
