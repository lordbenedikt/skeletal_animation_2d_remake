use crate::*;
use bone::Bone;

#[derive(Component)]
pub struct Target {
    pub bone: Entity,
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
) {
    // Add CCD Target only if Alt + Left Mouse was pressed
    if !keys.pressed(KeyCode::LAlt) || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    dbg!("add ccd");

    let mut opt_bone_entity: Option<Entity> = None;
    for (bone_entity, transformable) in q.iter() {
        if transformable.is_selected {
            opt_bone_entity = Some(bone_entity);
            break;
        }
    }
    if let Some(bone_entity) = opt_bone_entity {
        commands
            .spawn_bundle(TransformBundle::from_transform(
                Transform::default().with_translation(cursor_pos.0.extend(0.)),
            ))
            .insert(Target { bone: bone_entity })
            .insert(Transformable::default());
    }
    for (_, mut transformable) in q.iter_mut() {
        transformable.is_selected = false;
    }
    transform_state.action = Action::Done;
}

pub fn reach_for_target(
    mut commands: Commands,
    mut q_bones: Query<(&GlobalTransform, Option<&Parent>, &mut Transform), With<Bone>>,
    q_targets: Query<(Entity, &Transform, &Target), Without<Bone>>,
) {
    for (entity, target_transform, target) in q_targets.iter() {
        let depth = 2;
        let iterations = 5;
        // If bone was removed, despawn target
        if q_bones.get(target.bone).is_err() {
            commands.entity(entity).despawn();
            return;
        };

        // Get tip of last bone of chain
        let last_bone_gl_transform = q_bones.get(target.bone).unwrap().0.clone();
        let mut end_of_chain: Vec2 = Bone::get_tip(&last_bone_gl_transform);

        // Perform CCD
        for _ in 0..iterations {
            let mut current_bone: Entity = target.bone;
            for _ in 0..depth {
                // Rotate current bone so that current_pos, end_of_chain and target are on one line
                let current_pos = q_bones.get(current_bone).unwrap().0.translation.truncate();
                let delta_rot = Quat::from_rotation_arc(
                    (end_of_chain.extend(0.) - current_pos.extend(0.)).normalize(),
                    (target_transform.translation - current_pos.extend(0.)).normalize(),
                );
                // dbg!(delta_rot);
                q_bones.get_mut(current_bone).unwrap().2.rotation *= delta_rot;
                let end_of_chain_relative = end_of_chain - current_pos;
                let end_of_chain_relative_rotated =
                    Quat::mul_vec3(delta_rot, end_of_chain_relative.extend(0.)).truncate();
                end_of_chain = end_of_chain_relative_rotated + current_pos;

                // If parent exists, continue with parent
                if let Some(parent) = q_bones.get(target.bone).unwrap().1 {
                    current_bone = parent.0;
                } else {
                    break;
                }
            }
        }
    }
}
