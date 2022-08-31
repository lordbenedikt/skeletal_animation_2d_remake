use crate::animation::Animatable;
use crate::bone::Bone;
use crate::*;
use bevy::utils::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::Write;
use std::{fs, io::Error};

#[derive(Serialize, Deserialize, Clone)]
struct CompleteJson {
    bones: Vec<BoneJson>,
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

pub fn system_set() -> SystemSet {
    SystemSet::new().with_system(save).with_system(load)
}

fn save(q: Query<(Entity, &Bone, &Transform, Option<&Parent>)>, keys: Res<Input<KeyCode>>) {
    if keys.pressed(KeyCode::LControl) {
        let save_slot = get_just_pressed_number(keys);
        if save_slot == -1 {
            return;
        }
        let mut res = Entity::from_bits(0);
        let bones = q
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
        let serialized = serde_json::to_string(&CompleteJson { bones }).unwrap();
        let mut file = fs::File::create(format!("anims/animation_{}.json", save_slot)).unwrap();
        file.write_all(serialized.as_bytes()).unwrap();
        // dbg!(&serialized);
    }
}

// fn load(mut commands: Commands, keys: Res<Input<KeyCode>>) {

// // Spawn as child of parent
// let mut res = Entity::from_bits(0);
// let (parent_gl_transform, _, _, _) = q.get(parent).unwrap();

// let mut gl_transform = Transform {
//     scale: gl_scale,
//     rotation: gl_rotation,
//     translation: gl_translation,
// };

// gl_transform.scale.z = 1.;
// let transform = transform::get_relative_transform(parent_gl_transform, &gl_transform);

// commands.entity(parent).with_children(|parent| {
//     res = parent
//         .spawn_bundle(SpriteBundle {
//             sprite: Sprite {
//                 color: Color::rgb(0.4, 0.4, 0.4),
//                 ..Default::default()
//             },
//             transform,
//             visibility: Visibility {
//                 is_visible: show_sprite,
//             },
//             ..Default::default()
//         })
//         .insert(Bone::default())
//         .insert(Transformable::default())
//         .insert(Animatable)
//         .id();
// });
// res
// }

fn load_bone_recursive_no_parent(commands: &mut Commands, bones: &Vec<BoneJson>, index: usize) {
    let current_bone = bones[index].clone();
    commands
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
                    load_bone_recursive_with_parent(p, bones, i);
                }
            }
        });
}

fn load_bone_recursive_with_parent(parent: &mut ChildBuilder, bones: &Vec<BoneJson>, index: usize) {
    let current_bone = bones[index].clone();
    parent
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
                    load_bone_recursive_with_parent(p, bones, i);
                }
            }
        });
}

fn load(
    // asset_server: Res<AssetServer>,
    keys: Res<Input<KeyCode>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q: ParamSet<(Query<Entity, With<Bone>>, Query<(Entity, &skin::Skin)>)>,
    mut commands: Commands,
    mut transform_state: ResMut<transform::State>,
) {
    if keys.pressed(KeyCode::LAlt) {
        let save_slot = get_just_pressed_number(keys);
        if save_slot == -1 {
            return;
        }
        for entity in q.p0().iter() {
            commands.entity(entity).despawn();
        }
        for (entity,skin) in q.p1().iter() {
            commands.entity(entity).despawn();
            meshes.remove(skin.mesh_handle.clone().unwrap().0);
        }
        let json = fs::read_to_string(format!("anims/animation_{}.json", save_slot)).unwrap();
        let data: CompleteJson = serde_json::from_str(&json).unwrap();
        // let bones_to_spawn: Vec<BoneJson> = data.bones.iter().map(|b| (*b).clone()).collect();
        // let spawned_bones: HashMap<ID, Entity> = HashMap::new();
        for i in 0..data.bones.len() {
            if data.bones[i].parent.is_none() {
                load_bone_recursive_no_parent(&mut commands, &data.bones, i);
            }
        }

        transform_state.selected_entities.clear();
    }
    //         commands.entity(parent).with_children(|parent| {
    //             res = parent
    //                 .spawn_bundle(SpriteBundle {
    //                     sprite: Sprite::default(),
    //                     transform,
    //                     visibility: Visibility {
    //                         is_visible: show_sprite,
    //                     },
    //                     ..Default::default()
    //                 })
    //                 .insert(Bone::default())
    //                 .insert(Transformable::default())
    //                 .insert(Animatable)
    //                 .id();
    //         });
    //         res
    //     }
    //     commands
    //         .spawn_bundle(SpriteBundle {
    //             sprite: Sprite {
    //                 color: Color::rgb(0.4, 0.4, 0.4),
    //                 ..Default::default()
    //             },
    //             transform: Transform {
    //                 translation: current_bone.translation,
    //                 rotation: current_bone.rotation,
    //                 scale: current_bone.scale,
    //                 ..Default::default()
    //             },
    //             visibility: Visibility { is_visible: true },
    //             ..Default::default()
    //         })
    //         .insert(Bone::default())
    //         .insert(Transformable::default())
    //         .insert(Animatable)
    //         .id();
    // }
    // while !bones_to_spawn.is_empty() {
    //     for i in (0..bones_to_spawn.len()).rev() {
    //         if bone_json.parent.is_none() || spawned_bones.contains_key(bone_json.id) {
    //             // Spawn without parent
    // commands
    //     .spawn_bundle(SpriteBundle {
    //         sprite: Sprite {
    //             color: Color::rgb(0.4, 0.4, 0.4),
    //             ..Default::default()
    //         },
    //         transform: Transform {
    //             translation: Vec3::new(cursor_pos.0.x, cursor_pos.0.y, bone_depth),
    //             rotation: Quat::from_rotation_z(0.),
    //             scale: Vec3::new(1., 1., 1.),
    //             ..Default::default()
    //         },
    //         visibility: Visibility {
    //             is_visible: show_sprite,
    //         },
    //         ..Default::default()
    //     })
    //     .insert(Bone::default())
    //     .insert(Transformable::default())
    //     .insert(Animatable)
    //     .id();
    //             bones_to_spawn.remove(i);
    //         }
    //     }
    // }
    // }
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
