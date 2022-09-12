use crate::{animation::Animatable, *};
use bevy::math;
use bone::Bone;

#[derive(Component, Clone)]
pub struct Target {
    pub bone: Entity,
    pub depth: u8,
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(add_target)
        .with_system(reach_for_target)
}

pub fn add_target(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transformable), With<Bone>>,
    cursor_pos: Res<CursorPos>,
    keys: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut transform_state: ResMut<transform::State>,
    egui_state: Res<egui::State>,
    asset_server: Res<AssetServer>,
) {
    // Add CCD Target only if Alt + Left Mouse was pressed
    if !keys.pressed(KeyCode::LAlt) || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let mut opt_bone_entity: Option<Entity> = None;
    for (bone_entity, transformable) in q.iter() {
        if transformable.is_selected {
            opt_bone_entity = Some(bone_entity);
            break;
        }
    }
    if let Some(bone_entity) = opt_bone_entity {
        transform_state.selected_entities.clear();
        transform_state.selected_entities.insert(
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::default().with_translation(cursor_pos.0.extend(500.)),
                    sprite: Sprite {
                        color: COLOR_DEFAULT,
                        custom_size: Some(Vec2::new(0.4, 0.4)),
                        ..Default::default()
                    },
                    texture: asset_server.load("img/ccd_target.png"),
                    ..Default::default()
                })
                .insert(Target {
                    bone: bone_entity,
                    depth: egui_state.ccd_depth,
                })
                .insert(Transformable::default())
                .insert(Animatable)
                .id(),
        );
    } else {
        return;
    }
    for (_, mut transformable) in q.iter_mut() {
        transformable.is_selected = false;
    }
    transform_state.action = Action::Done;
}

pub fn reach_for_target(
    mut commands: Commands,
    mut q_bones: Query<(&GlobalTransform, Option<&Parent>, &mut Transform, &mut Bone)>,
    q_targets: Query<(Entity, &Transform, &Target), Without<Bone>>,
) {
    // Reset bone.is_ccd_maneuvered
    for (_,_,_,mut bone) in q_bones.iter_mut() {
        bone.is_ccd_maneuvered = false;
    }
    
    for (entity, target_transform, target) in q_targets.iter() {
        let depth = target.depth;
        let iterations = 20;
        // If bone was removed, despawn target
        if q_bones.get(target.bone).is_err() {
            commands.entity(entity).despawn();
            return;
        };

        // Had to write this part twice for some reason, NOT PRETTY!!, need to look into finding a better solution
        // Manually propagate transform to always have global transform that is up to date
        let get_true_gl_transform = |entity: Entity| -> Transform {
            let mut current_entity: Entity = entity;
            let mut res = q_bones.get(entity).unwrap().2.clone();
            // While parent exists, combine with parent's transform
            while let Some(parent) = q_bones.get(current_entity).unwrap().1.clone() {
                current_entity = parent.get();
                res = transform::combined_transform(
                    q_bones.get(current_entity).unwrap().2.clone(),
                    res,
                );
            }
            res
        };

        // Get tip of last bone of chain
        let last_bone_gl_transform = q_bones.get(target.bone).unwrap().0.clone();
        let mut end_of_chain: Vec2 = Bone::get_true_tip(&get_true_gl_transform(target.bone));
        // let mut end_of_chain: Vec2 = Bone::get_tip(&last_bone_gl_transform);

        // Perform CCD
        for _ in 0..iterations {
            let mut current_bone: Entity = target.bone;
            for _ in 0..depth {
                // Set bone's ccd_maneuvered to true
                q_bones.get_mut(current_bone).unwrap().3.is_ccd_maneuvered = true;

                // Rotate current bone so that current_pos, end_of_chain and target are on one line
                let current_pos = q_bones
                    .get(current_bone)
                    .unwrap()
                    .0
                    .affine()
                    .translation
                    .truncate();

                let delta_rot = (end_of_chain - current_pos).get_angle()
                    - (target_transform.translation.truncate() - current_pos).get_angle();

                // Store values before change
                let original_rotation = q_bones.get_mut(current_bone).unwrap().2.rotation.clone();
                let original_end_of_chain = end_of_chain;
                let original_distance = Vec2::distance(
                    target_transform.translation.truncate(),
                    original_end_of_chain,
                );

                q_bones.get_mut(current_bone).unwrap().2.rotation *= //delta_rot;
                Quat::from_rotation_z(delta_rot);

                // Manually propagate transform to always have global transform that is up to date
                let get_true_gl_transformmm = |entity: Entity| -> Transform {
                    let mut current_entity: Entity = entity;
                    let mut res = q_bones.get(entity).unwrap().2.clone();
                    // While parent exists, combine with parent's transform
                    while let Some(parent) = q_bones.get(current_entity).unwrap().1.clone() {
                        current_entity = parent.get();
                        res = transform::combined_transform(
                            q_bones.get(current_entity).unwrap().2.clone(),
                            res,
                        );
                    }
                    res
                };

                let end_of_chain_relative = end_of_chain - current_pos;
                let end_of_chain_relative_rotated = end_of_chain_relative.rotate_by(delta_rot);
                end_of_chain = Bone::get_true_tip(&get_true_gl_transformmm(target.bone));

                // If new rotation didn't bring improvement, undo
                let new_distance =
                    Vec2::distance(target_transform.translation.truncate(), end_of_chain);
                if new_distance >= original_distance {
                    q_bones.get_mut(current_bone).unwrap().2.rotation = original_rotation;
                    end_of_chain = original_end_of_chain;
                }

                // If parent exists, continue with parent
                if let Some(parent) = q_bones.get(current_bone).unwrap().1 {
                    current_bone = parent.get();
                } else {
                    break;
                }
            }
        }
    }
}

trait Vec2Angles {
    fn get_angle(self) -> f32;
    fn rotate_by(self, angle: f32) -> Self;
}

impl Vec2Angles for Vec2 {
    fn get_angle(self) -> f32 {
        let angle = self.angle_between(Vec2::Y);
        angle
    }
    fn rotate_by(self, angle: f32) -> Self {
        Vec2::from_angle(angle).rotate(self)
        // let x = self.x * angle.cos() + self.y * (-angle.sin());
        // let y = self.x * angle.sin() + self.y * angle.cos();
        // Vec2::new(x, y)
    }
}
