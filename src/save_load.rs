use crate::animation::{Animatable, Animation, Animations, ComponentAnimation};
use crate::bone::Bone;
use crate::ccd::Target;
use crate::cloth::Cloth;
use crate::skeleton::{Skeleton, SkinMapping};
use crate::skin::Skin;
use crate::*;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::{fs, io::Error};

#[derive(Serialize, Deserialize, Clone)]
struct CompleteJson {
    skeleton: SkeletonJson,
    animations: AnimationsJson,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationsJson {
    map: HashMap<String, AnimationJson>,
}
impl AnimationsJson {
    fn from_animations(anims: &Animations) -> Self {
        Self {
            map: anims
                .map
                .iter()
                .map(|(key, value)| (key.clone(), AnimationJson::from_animation(value)))
                .collect(),
        }
    }
    fn as_animations(&self, spawned_entities: &HashMap<Entity, Entity>) -> Animations {
        Animations {
            map: self
                .map
                .iter()
                .map(|(key, value)| {
                    (
                        key.clone(),
                        AnimationJson::as_animation(value, spawned_entities),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationJson {
    keyframes: Vec<f64>,
    comp_animations: HashMap<Entity, ComponentAnimationJson>,
}
impl AnimationJson {
    fn from_animation(anim: &Animation) -> Self {
        Self {
            keyframes: anim.keyframes.clone(),
            comp_animations: {
                let mut res: HashMap<Entity, ComponentAnimationJson> = HashMap::new();
                for (&key, value) in anim.comp_animations.iter() {
                    res.insert(key, ComponentAnimationJson::from_component_animation(value));
                }
                res
            },
        }
    }
    fn as_animation(&self, spawned_entities: &HashMap<Entity, Entity>) -> Animation {
        Animation {
            keyframes: self.keyframes.clone(),
            comp_animations: {
                let mut res: HashMap<Entity, ComponentAnimation> = HashMap::new();
                for (&key, value) in self.comp_animations.iter() {
                    if spawned_entities.get(&key).is_none() {
                        continue;
                    }
                    res.insert(
                        *spawned_entities.get(&key).unwrap(),
                        ComponentAnimationJson::as_component_animation(value),
                    );
                }
                res
            },
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ComponentAnimationJson {
    keyframe_indices: Vec<usize>,
    translations: Vec<Vec3>,
    scales: Vec<Vec3>,
    rotations: Vec<Quat>,
    interpolation_functions: Vec<interpolate::Function>,
}
impl ComponentAnimationJson {
    fn from_component_animation(comp_anim: &ComponentAnimation) -> Self {
        let mut translations: Vec<Vec3> = vec![];
        let mut scales: Vec<Vec3> = vec![];
        let mut rotations: Vec<Quat> = vec![];
        for transform in comp_anim.transforms.iter() {
            translations.push(transform.translation);
            scales.push(transform.scale);
            rotations.push(transform.rotation);
        }
        Self {
            keyframe_indices: comp_anim.keyframe_indices.clone(),
            translations,
            scales,
            rotations,
            interpolation_functions: comp_anim.interpolation_functions.clone(),
        }
    }
    fn as_component_animation(&self) -> ComponentAnimation {
        let mut transforms = vec![];
        for i in 0..self.translations.len() {
            transforms.push(Transform {
                translation: self.translations[i],
                scale: self.scales[i],
                rotation: self.rotations[i],
            });
        }
        ComponentAnimation {
            keyframe_indices: self.keyframe_indices.clone(),
            transforms,
            interpolation_functions: self.interpolation_functions.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct SkeletonJson {
    bones: Vec<BoneJson>,
    skins: Vec<SkinJson>,
    targets: Vec<TargetJson>,
    skin_mappings: Vec<SkinMapping>,
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Clone)]
struct ID(Entity);

#[derive(Serialize, Deserialize, Clone)]
struct BoneJson {
    entity: Entity,
    parent: Option<Entity>,
    translation: Vec3,
    scale: Vec3,
    rotation: Quat,
}
impl PartialEq for BoneJson {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SkinJson {
    entity: Entity,
    filename: String,
    dimensions: [u32; 2],
    vertices: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u16>,
    depth: f32,
    cloth: Option<Cloth>,
}
impl SkinJson {
    fn as_skin(&self) -> Skin {
        Skin {
            filename: self.filename.clone(),
            dimensions: self.dimensions.clone(),
            vertices: self.vertices.clone(),
            uvs: self.uvs.clone(),
            indices: self.indices.clone(),
            mesh_handle: None,
            depth: self.depth,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct TargetJson {
    entity: Entity,
    bone: Entity,
    depth: u8,
    translation: Vec3,
}

pub fn system_set() -> SystemSet {
    SystemSet::new().with_system(save).with_system(load)
}

fn save(
    mut set: ParamSet<(
        Query<(Entity, &Bone, &Transform, Option<&Parent>)>,
        Query<(Entity, &Skin, Option<&Cloth>)>,
        Query<(Entity, &Target, &Transform)>,
    )>,
    animations: Res<Animations>,
    skeleton: Res<Skeleton>,
    keys: Res<Input<KeyCode>>,
) {
    if keys.pressed(KeyCode::LControl) {
        let save_slot = get_just_pressed_number(keys);
        if save_slot == -1 {
            return;
        }
        let mut res = Entity::from_bits(0);
        let bones = set
            .p0()
            .iter()
            .map(|(entity, bone, transform, opt_parent)| BoneJson {
                entity,
                parent: if let Some(parent) = opt_parent {
                    Some(parent.get())
                } else {
                    None
                },
                translation: transform.translation,
                scale: transform.scale,
                rotation: transform.rotation,
            })
            .collect::<Vec<BoneJson>>();
        let skins = set
            .p1()
            .iter()
            .map(|(entity, skin, opt_cloth)| SkinJson {
                entity,
                filename: skin.filename.clone(),
                dimensions: skin.dimensions,
                uvs: skin.uvs.clone(),
                vertices: skin.vertices.clone(),
                indices: skin.indices.clone(),
                depth: skin.depth,
                cloth: if let Some(cloth) = opt_cloth {
                    Some(cloth.clone())
                } else {
                    None
                },
            })
            .collect::<Vec<SkinJson>>();
        let targets = set
            .p2()
            .iter()
            .map(|(entity, target, transform)| TargetJson {
                entity,
                bone: target.bone,
                depth: target.depth,
                translation: transform.translation,
            })
            .collect::<Vec<TargetJson>>();
        let serialized = serde_json::to_string(&CompleteJson {
            skeleton: SkeletonJson {
                bones,
                skins,
                targets,
                skin_mappings: skeleton.skin_mappings.clone(),
            },
            animations: AnimationsJson::from_animations(&animations),
        })
        .unwrap();
        let mut file = fs::File::create(format!("anims/animation_{}.json", save_slot)).unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
    }
}

fn load_bone_recursive_no_parent(
    commands: &mut Commands,
    bones: &Vec<BoneJson>,
    spawned_entities: &mut HashMap<Entity, Entity>,
    index: usize,
) {
    let current_bone = bones[index].clone();
    let bone_entity = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::default(),
            transform: Transform {
                translation: current_bone.translation,
                rotation: current_bone.rotation,
                scale: current_bone.scale,
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Bone::default())
        .insert(Transformable {
            is_selected: false,
            ..Default::default()
        })
        .insert(Animatable)
        .with_children(|p| {
            for i in 0..bones.len() {
                if bones[i].parent.is_some() && bones[i].parent.unwrap() == current_bone.entity {
                    load_bone_recursive_with_parent(p, bones, spawned_entities, i);
                }
            }
        })
        .id();
    spawned_entities.insert(current_bone.entity, bone_entity);
}

fn load_bone_recursive_with_parent(
    parent: &mut ChildBuilder,
    bones: &Vec<BoneJson>,
    spawned_bones: &mut HashMap<Entity, Entity>,
    index: usize,
) {
    let current_bone = bones[index].clone();
    let bone_entity = parent
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::default(),
            transform: Transform {
                translation: current_bone.translation,
                rotation: current_bone.rotation,
                scale: current_bone.scale,
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(Bone::default())
        .insert(Transformable {
            is_selected: false,
            ..Default::default()
        })
        .insert(Animatable)
        .with_children(|p| {
            for i in 0..bones.len() {
                if bones[i].parent.is_some() && bones[i].parent.unwrap() == current_bone.entity {
                    load_bone_recursive_with_parent(p, bones, spawned_bones, i);
                }
            }
        })
        .id();
    spawned_bones.insert(current_bone.entity, bone_entity);
}

fn load(
    // asset_server: Res<AssetServer>,
    keys: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut skeleton: ResMut<Skeleton>,
    mut q: ParamSet<(
        Query<Entity, With<Bone>>,
        Query<(Entity, &skin::Skin)>,
        Query<(Entity, &Target)>,
    )>,
    mut commands: Commands,
    mut animations: ResMut<Animations>,
    mut transform_state: ResMut<transform::State>,
    mut egui_state: ResMut<egui::State>,
    mut anim_state: ResMut<animation::State>,
) {
    if keys.pressed(KeyCode::LAlt) {
        let save_slot = get_just_pressed_number(keys);
        if save_slot == -1 {
            return;
        }
        for entity in q.p0().iter() {
            commands.entity(entity).despawn();
        }
        for (entity, skin) in q.p1().iter() {
            commands.entity(entity).despawn();
            meshes.remove(skin.mesh_handle.clone().unwrap().0);
        }
        for (entity, target) in q.p2().iter() {
            commands.entity(entity).despawn();
        }
        let json = fs::read_to_string(format!("anims/animation_{}.json", save_slot)).unwrap();
        let mut data: CompleteJson = serde_json::from_str(&json).unwrap();

        // Json ID to spawned entity ID, necessary because Game Engines assigns IDs automatically
        let mut spawned_entities: HashMap<Entity, Entity> = HashMap::new();

        // Spawn Bones
        for i in 0..data.skeleton.bones.len() {
            if data.skeleton.bones[i].parent.is_none() {
                load_bone_recursive_no_parent(
                    &mut commands,
                    &data.skeleton.bones,
                    &mut spawned_entities,
                    i,
                );
            }
        }
        // Spawn Skins
        for i in 0..data.skeleton.skins.len() {
            let mut skin = data.skeleton.skins[i].as_skin();

            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skin.vertices.clone());
            mesh.insert_attribute(
                Mesh::ATTRIBUTE_NORMAL,
                vec![[0., 0., 1.]; skin.vertices.len()],
            );
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, skin.uvs.clone());
            mesh.set_indices(Some(Indices::U16(skin.indices.clone())));

            let handle: Mesh2dHandle = meshes.add(mesh).into();
            skin.mesh_handle = Some(handle.clone());

            commands.spawn_bundle(MaterialMesh2dBundle {
                mesh: handle.clone(),
                material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
                ..default()
            });
            let skin_entity = commands
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
            if let Some(cloth) = data.skeleton.skins[i].cloth.clone() {
                commands.entity(skin_entity).insert(cloth);
            }
            spawned_entities.insert(
                data.skeleton.skins[i].entity,
                commands.entity(skin_entity).id(),
            );
        }

        // Spawn Targets
        for i in 0..data.skeleton.targets.len() {
            let target = data.skeleton.targets[i].clone();
            let target_entity = commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::from_translation(target.translation),
                    sprite: Sprite {
                        color: COLOR_DEFAULT,
                        custom_size: Some(Vec2::new(0.4, 0.4)),
                        ..Default::default()
                    },
                    texture: asset_server.load("img/ccd_target.png"),
                    ..Default::default()
                })
                .insert(Target {
                    bone: *spawned_entities.get(&target.bone).unwrap(),
                    depth: target.depth,
                })
                .insert(Transformable::default())
                .insert(Animatable)
                .id();
            spawned_entities.insert(target.entity, target_entity);
        }

        // Build Skeleton
        skeleton.bones = spawned_entities.values().into_iter().map(|&e| e).collect();
        for skin_mapping in data.skeleton.skin_mappings.iter_mut() {
            if let Some(json_entity) = skin_mapping.skin {
                let opt_new_entity = spawned_entities.get(&json_entity);
                skin_mapping.skin = if let Some(new_entity) = opt_new_entity {
                    Some(*new_entity)
                } else {
                    None
                };
            }
            for vertex_mapping in skin_mapping.vertex_mappings.iter_mut() {
                for i in (0..vertex_mapping.bones.len()).rev() {
                    if spawned_entities.get(&vertex_mapping.bones[i]).is_none() {
                        vertex_mapping.weights.swap_remove(i);
                        vertex_mapping.bones.swap_remove(i);
                        vertex_mapping.rel_positions.swap_remove(i);
                        continue;
                    }
                    vertex_mapping.bones[i] =
                        *spawned_entities.get(&vertex_mapping.bones[i]).unwrap();
                }
            }
        }
        skeleton.skin_mappings = data.skeleton.skin_mappings;

        // Clear Selection
        transform_state.selected_entities.clear();

        // Load Animations
        animations.map = data.animations.as_animations(&spawned_entities).map;

        // Select first Animation
        egui_state.plots[0].name = if let Some(name) = animations.map.keys().next() {
            name.clone()
        } else {
            String::new()
        };

        // Remove layers
        anim_state.layers.clear();
        anim_state.layers.push(String::from("anim_0"));
    }
}

fn get_just_pressed_number(keys: Res<Input<KeyCode>>) -> i32 {
    if keys.just_pressed(KeyCode::Key1) {
        return 1;
    } else if keys.just_pressed(KeyCode::Key2) {
        return 2;
    } else if keys.just_pressed(KeyCode::Key3) {
        return 3;
    } else if keys.just_pressed(KeyCode::Key4) {
        return 4;
    } else if keys.just_pressed(KeyCode::Key5) {
        return 5;
    } else if keys.just_pressed(KeyCode::Key6) {
        return 6;
    } else if keys.just_pressed(KeyCode::Key7) {
        return 7;
    } else if keys.just_pressed(KeyCode::Key8) {
        return 8;
    } else if keys.just_pressed(KeyCode::Key9) {
        return 9;
    } else if keys.just_pressed(KeyCode::Key0) {
        return 0;
    } else {
        return -1;
    }
}
