use crate::*;

#[derive(Component)]
pub struct Bone;

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(add_bone)
        .with_system(draw_debug_lines)
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
        let v_diff =
            Vec3::new(cursor_pos.0.x, cursor_pos.0.y, bone_depth) - parent_gl_transform.translation;
        let rel_translation = Quat::mul_vec3(Quat::inverse(parent_gl_transform.rotation), v_diff)
            / Vec3::new(parent_gl_transform.scale.x, parent_gl_transform.scale.y, 1.);
        commands.entity(parent).with_children(|parent| {
            res = parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.4, 0.4, 0.4),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: rel_translation,
                        rotation: Quat::from_rotation_z(0.),
                        scale: Vec3::new(1., 1., 0.),
                        ..Default::default()
                    },
                    visibility: Visibility {
                        is_visible: show_sprite,
                    },
                    ..Default::default()
                })
                .insert(Bone{})
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
                    scale: Vec3::new(1., 1., 0.),
                    ..Default::default()
                },
                visibility: Visibility {
                    is_visible: show_sprite,
                },
                ..Default::default()
            })
            .insert({dbg!("added");Bone{}})
            .insert(Transformable::default())
            .id()
    };
    skeleton.bones.push(entity);
    // Unselect all transformables
    for (_,_,_,mut transformable) in q.iter_mut() {
        transformable.is_selected = false;
    }
    state.action = Action::Done;
}

pub fn draw_debug_lines(
    mut debug_drawer: ResMut<DebugDrawer>,
    bone_gl_transforms: Query<(&GlobalTransform, &Bone, &Transformable)>,
) {
    for (gl_transform, bone,transformable) in bone_gl_transforms.iter() {
        let z = 0.001;
        let scale = gl_transform.scale;
        let mut points = vec![
            Vec3::new(0., 0., z),
            Vec3::new(-0.1, 0.1, z),
            Vec3::new(0., 1., z),
            Vec3::new(0.1, 0.1, z),
            Vec3::new(0., 0., z),
        ];
        for i in 0..points.len() {
            points[i].x *= scale.x;
            points[i].y *= scale.y;
        }
        for i in 0..points.len() {
            debug_drawer.line(
                (gl_transform.translation + Quat::mul_vec3(gl_transform.rotation, points[i]))
                    .truncate(),
                (gl_transform.translation
                    + Quat::mul_vec3(gl_transform.rotation, points[(i + 1) % points.len()]))
                .truncate(),
                if transformable.is_selected {
                    COLOR_SELECTED
                } else {
                    COLOR_DEFAULT
                },
            );
        }
    }
}
