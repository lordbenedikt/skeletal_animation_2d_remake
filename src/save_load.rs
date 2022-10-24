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
use std::fs;
use std::io::Write;
use wasm_bindgen::JsValue;

#[derive(Default)]
pub struct State {
    pub opt_load_path: Option<String>,
    pub load_count: i32,
}

#[derive(Serialize, Deserialize, Clone, bevy::reflect::TypeUuid)]
#[uuid = "413be529-bfeb-41b3-9db0-4b8b380a2c12"]
pub struct CompleteJson {
    skeleton: SkeletonJson,
    animations: AnimationsJson,
    animation_layers: Vec<String>,
    blending_style: animation::BlendingStyle,
}

pub struct SaveEvent(pub Option<i32>);

pub struct LoadEvent(CompleteJson);

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
    vertices: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Vec<u16>,
    depth: f32,
    cloth: Option<Cloth>,
}
impl SkinJson {
    fn as_skin(&self) -> Skin {
        Skin {
            path: self.filename.clone(),
            vertices: self.vertices.clone(),
            uvs: self.uvs.clone(),
            indices: self.indices.clone(),
            mesh_handle: None,
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
    SystemSet::new()
        .with_system(load)
        .with_system(call_load_event)
        .with_system(save)
        .with_system(call_save_event)
}

fn call_save_event(keys: Res<Input<KeyCode>>, mut save_evw: EventWriter<SaveEvent>) {
    #[cfg(not(target_arch = "wasm32"))]
    if !keys.pressed(KeyCode::LControl) {
        let save_slot = get_just_pressed_number(&keys);
        if save_slot == -1 {
            return;
        }
        save_evw.send(SaveEvent(Some(save_slot)));
    }
}

fn save(
    mut set: ParamSet<(
        Query<(Entity, &Transform, Option<&Parent>), With<Bone>>,
        Query<(Entity, &Skin, Option<&Cloth>)>,
        Query<(Entity, &Target, &Transform)>,
    )>,
    animations: Res<Animations>,
    anim_state: Res<animation::State>,
    skeleton: Res<Skeleton>,
    mut save_evr: EventReader<SaveEvent>,
) {
    for e in save_evr.iter() {
        let opt_save_slot = e.0;
        let bones = set
            .p0()
            .iter()
            .map(|(entity, transform, opt_parent)| BoneJson {
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
                filename: skin.path.clone(),
                uvs: skin.uvs.clone(),
                vertices: skin.vertices.clone(),
                indices: skin.indices.clone(),
                depth: 0.,
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
            animation_layers: anim_state.layers.clone(),
            blending_style: anim_state.blending_style,
        })
        .unwrap();
        save_to_file(&serialized, opt_save_slot);
    }
}

fn save_to_file(serialized: &str, save_slot: Option<i32>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut file = fs::File::create(format!(
            "assets/anims/animation_{}.anim",
            save_slot.unwrap()
        ))
        .expect("Failed to create file!");
        file.write_all(serialized.as_bytes()).unwrap();
    }

    // If on web, download anim-file
    #[cfg(target_arch = "wasm32")]
    {
        let document = web_sys::window().unwrap().document().unwrap();
        let element = document.create_element("a").unwrap();

        element.set_attribute(
            "href",
            &format!(
                "data:text/plain;charset=utf-8,{}",
                js_sys::encode_uri_component(&serialized)
            ),
        );
        element.set_attribute("download", &format!("my_animation.anim"));

        let event = document.create_event("MouseEvents").unwrap();
        event.init_event("click");
        element.dispatch_event(&event);
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

fn call_load_event(
    keys: Res<Input<KeyCode>>,
    mut load_evw: EventWriter<LoadEvent>,
    savefile_assets: Res<Assets<CompleteJson>>,
    asset_server: Res<AssetServer>,
    mut state: ResMut<State>,
) {
    // #[cfg(not(target_arch = "wasm32"))]
    {
        let save_slot = get_just_pressed_number(&keys);
        if keys.pressed(KeyCode::LAlt) && save_slot != -1 {
            state.opt_load_path = Some(anim_name_to_path(&format!("animation_{}", save_slot)));
        }

        if let Some(path) = &state.opt_load_path {
            let anim_handle = asset_server.load(path);
            let opt_data = savefile_assets.get(&anim_handle);

            if let Some(data) = opt_data {
                load_evw.send(LoadEvent(data.clone()));
                state.opt_load_path = None
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        if let Some(count) = local_storage.get("load_count").unwrap() {
            let count_i32 = count.parse::<i32>().unwrap();
            if count_i32 != state.load_count {
                state.load_count = count_i32;
                let data_string = local_storage.get("loaded_anim").unwrap().unwrap();
                let data = serde_json::from_str::<CompleteJson>(&data_string).unwrap();
                load_evw.send(LoadEvent(data));
            }
        }
    }

}

fn load(
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
    mut load_evr: EventReader<LoadEvent>,
) {
    for e in load_evr.iter() {
        let mut data = e.0.clone();

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
                material: materials.add(ColorMaterial::from(asset_server.load(&skin.path))),
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

        // Load Layers
        anim_state.layers = data.animation_layers;

        // Load Blending Style
        anim_state.blending_style = data.blending_style;

        // Select first Animation
        egui_state.plots[0].name = if let Some(name) = animations.map.keys().next() {
            name.clone()
        } else {
            String::new()
        };
    }
}

fn get_just_pressed_number(keys: &Input<KeyCode>) -> i32 {
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

pub fn anim_name_to_path(filename: &str) -> String {
    String::from(format!("anims/{}.anim", filename))
}
