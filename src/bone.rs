use crate::*;

#[derive(Component)]
pub struct Bone;
impl Bone {
    pub fn get_tip(gl_transform: &GlobalTransform) -> Vec2 {
        let mut res = gl_transform.translation;
        res += Quat::mul_vec3(
            gl_transform.rotation,
            Vec3::new(0., gl_transform.scale.y, 0.),
        );
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
    mut state: ResMut<state::State>,
    mut skeleton: ResMut<skeleton::Skeleton>,
) {
    let show_sprite = false;
    // Return if action is already taken
    if state.action != Action::None {
        return;
    }
    // Add bone only if CTRL and LEFT MOUSE was pressed
    if !mouse.just_pressed(MouseButton::Left) || !keys.pressed(KeyCode::LControl) {
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
        let v_diff = Vec3::new(cursor_pos.0.x, cursor_pos.0.y, 0.) - gl_translation; // Vector representing new bone's protrusion
        let length = v_diff.length();
        let gl_scale = Vec3::new(length, length, 1.);
        let gl_rotation = Quat::from_rotation_arc(Vec3::new(0., 1., 0.).normalize(), v_diff.normalize());
        // let translation = Quat::mul_vec3(Quat::inverse(parent_gl_transform.rotation), v_diff)
        //     / Vec3::new(parent_gl_transform.scale.x, parent_gl_transform.scale.y, 1.);
        let gl_transform = GlobalTransform {
            translation: gl_translation,
            rotation: gl_rotation,
            scale: gl_scale,
        };
        let transform = transform::get_relative_transform(
            parent_gl_transform,
            &gl_transform,
        );

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
                .insert(Bone {})
                .insert(Transformable::default())
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
            .insert({
                dbg!("added");
                Bone {}
            })
            .insert(Transformable::default())
            .id()
    };
    skeleton.bones.push(entity);
    // Unselect all transformables
    for (_, _, _, mut transformable) in q.iter_mut() {
        transformable.is_selected = false;
    }
    state.action = Action::Done;
}
