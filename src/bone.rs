use crate::{animation::Animatable, skeleton::Skeleton, *};

#[derive(Default, Clone)]
pub struct AngleConstraint {
    pub start: f32,
    pub end: f32,
}

#[derive(Component)]
pub struct Bone {
    pub is_ik_maneuvered: bool,
    pub ik_angle_constraint: Option<AngleConstraint>,
}
impl Bone {
    pub fn get_tip(gl_transform: &GlobalTransform) -> Vec2 {
        let (scale, rotation, translation) = gl_transform.to_scale_rotation_translation();
        let mut res = translation;
        res += rotation.mul_vec3(Vec3::new(0., scale.y, 0.));
        res.truncate()
    }
    pub fn get_tip_global(gl_transform: &Transform) -> Vec2 {
        let mut res = gl_transform.translation;
        res += gl_transform
            .rotation
            .mul_vec3(Vec3::new(0., gl_transform.scale.y, 0.));
        res.truncate()
    }
}
impl Default for Bone {
    fn default() -> Self {
        Self {
            is_ik_maneuvered: false,
            ik_angle_constraint: Some(AngleConstraint::default()),
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new().with_system(add_bone_on_mouse_click)
}

pub fn add_bone_on_mouse_click(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    egui_state: Res<egui::State>,
    mut q: Query<(
        &GlobalTransform,
        Option<&mut Bone>,
        Entity,
        &mut Transformable,
    )>,
    mut transform_state: ResMut<transform::State>,
    mut skeleton: ResMut<Skeleton>,
) {
    // Return if action is already taken
    if transform_state.action != Action::None || egui_state.ui_hover {
        return;
    }
    // Add bone only if CTRL and LEFT MOUSE was pressed
    if !mouse.just_released(MouseButton::Left) || !keys.pressed(KeyCode::LControl) {
        return;
    }
    let bone_depth = 0.1;
    let mut opt_parent: Option<Entity> = None;
    for (_, opt_bone, entity, transformable) in q.iter() {
        if transformable.is_selected && opt_bone.is_some() {
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
        let gl_rotation =
            Quat::from_rotation_arc(Vec3::new(0., 1., 0.).normalize(), v_diff.normalize());

        let mut gl_transform = Transform {
            scale: gl_scale,
            rotation: gl_rotation,
            translation: gl_translation,
        };

        gl_transform.scale.z = 1.;

        let transform = transform::get_relative_transform(&parent_gl_transform.as_transform(), &gl_transform);

        commands.entity(parent).with_children(|parent| {
            res = parent
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: Color::rgb(0.4, 0.4, 0.4),
                        ..Default::default()
                    },
                    transform,
                    visibility: Visibility { is_visible: false },
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
                sprite: Sprite::default(),
                transform: Transform {
                    translation: Vec3::new(cursor_pos.0.x, cursor_pos.0.y, bone_depth),
                    rotation: Quat::from_rotation_z(0.),
                    scale: Vec3::new(1., 1., 1.),
                    ..Default::default()
                },
                visibility: Visibility { is_visible: false },
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

    // Clear selection_entities and push new entity
    transform_state.selected_entities.clear();
    transform_state.selected_entities.insert(entity);
}

pub fn get_bone_gl_transform(
    bone_entity: Entity,
    query: &Query<(&Transform, Option<&Parent>), With<Bone>>,
) -> Option<Transform> {
    let mut bone_gl_transform = Transform::default();
    let mut next_bone = bone_entity;
    loop {
        if let Ok((&bone_transform, opt_parent)) = query.get(next_bone) {
            bone_gl_transform = combined_transform(&bone_transform, &bone_gl_transform);
            if let Some(parent) = opt_parent {
                next_bone = parent.get();
            } else {
                break;
            }
        } else {
            return None;
        }
    }
    Some(bone_gl_transform)
}
