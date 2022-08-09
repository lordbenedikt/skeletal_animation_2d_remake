use crate::{animation::Animatable, *};

#[derive(Component, Default)]
pub struct Bone {
    pub is_ccd_maneuvered: bool,
}
impl Bone {
    pub fn get_tip(gl_transform: &GlobalTransform) -> Vec2 {
        let (scale, rotation, translation) = gl_transform.to_scale_rotation_translation();
        let mut res = translation;
        res += rotation.mul_vec3(Vec3::new(0., scale.y, 0.));
        res.truncate()
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new().with_system(add_bone)
}

pub fn add_bone(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, &mut Bone, Entity, &mut Transformable)>,
    mut transform_state: ResMut<transform::State>,
    mut skeleton: ResMut<skeleton::Skeleton>,
) {
    let show_sprite = false;
    // Return if action is already taken
    if transform_state.action != Action::None {
        return;
    }
    // Add bone only if CTRL and LEFT MOUSE was pressed
    if !mouse.just_released(MouseButton::Left) || !keys.pressed(KeyCode::LControl) {
        return;
    }
    let bone_depth = 0.1;
    let mut opt_parent: Option<Entity> = None;
    for (_, _, entity, transformable) in q.iter() {
        if transformable.is_selected {
            opt_parent = Some(entity);
            break;
        }
    }
    let entity = if let Some(parent) = opt_parent {
        // Spawn as child of parent
        let mut res = Entity::from_bits(0);
        let (parent_gl_transform, _, _, _) = q.get(parent).unwrap();

        let gl_translation = Bone::get_tip(parent_gl_transform).extend(0.); // New bones global transform
                                                                            // dbg!(&parent_gl_transform);
        let v_diff = Vec3::new(cursor_pos.0.x, cursor_pos.0.y, 0.) - gl_translation; // Vector representing new bone's protrusion
        let length = v_diff.length();
        let gl_scale = Vec3::new(length, length, 1.);
        let gl_rotation =
            Quat::from_rotation_arc(Vec3::new(0., 1., 0.).normalize(), v_diff.normalize());
        // let translation = Quat::mul_vec3(Quat::inverse(parent_gl_transform.rotation), v_diff)
        //     / Vec3::new(parent_gl_transform.scale.x, parent_gl_transform.scale.y, 1.);
        let mut gl_transform = Transform {
            scale: gl_scale,
            rotation: gl_rotation,
            translation: gl_translation,
        };
        // dbg!(&parent_gl_transform);
        // dbg!(&parent_gl_transform.rotation.to_euler(EulerRot::XYZ));
        // dbg!(&gl_transform);
        // dbg!(cursor_pos.0);
        // dbg!(gl_transform.rotation.to_euler(EulerRot::XYZ));
        // println!();
        gl_transform.scale.z = 1.;
        let transform = transform::get_relative_transform(parent_gl_transform, &gl_transform);

        commands.entity(parent).with_children(|parent| {
            res = parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.4, 0.4, 0.4),
                        ..Default::default()
                    },
                    transform,
                    visibility: Visibility {
                        is_visible: show_sprite,
                    },
                    ..Default::default()
                })
                .insert(Bone::default())
                .insert(Transformable::default())
                .insert(Animatable)
                .id();
        });
        res
    } else {
        // Spawn without parent
        commands
            .spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.4, 0.4, 0.4),
                    ..Default::default()
                },
                transform: Transform {
                    translation: Vec3::new(cursor_pos.0.x, cursor_pos.0.y, bone_depth),
                    rotation: Quat::from_rotation_z(0.),
                    scale: Vec3::new(1., 1., 1.),
                    ..Default::default()
                },
                visibility: Visibility {
                    is_visible: show_sprite,
                },
                ..Default::default()
            })
            .insert(Bone::default())
            .insert(Transformable::default())
            .insert(Animatable)
            .id()
    };
    skeleton.bones.push(entity);
    // Unselect all transformables
    for (_, _, _, mut transformable) in q.iter_mut() {
        transformable.is_selected = false;
    }
    transform_state.action = Action::Done;
}
