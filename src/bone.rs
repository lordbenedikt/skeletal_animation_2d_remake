use crate::*;

pub struct State {
    action: Action,
    cursor_anchor: Vec2,
    original_transforms: Vec<Transform>,
    selected_entities: Vec<Entity>,
}
impl State {
    pub fn new() -> State {
        State {
            action: Action::None,
            cursor_anchor: Vec2::new(0., 0.),
            original_transforms: vec![],
            selected_entities: vec![],
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Action {
    None,
    Translate,
    Rotate,
    Scale,
    Done,
}

#[derive(Component)]
pub struct Bone {
    is_selected: bool,
}
impl Bone {
    fn new() -> Bone {
        Bone { is_selected: true }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(start_action)
        .with_system(transform_bone)
        .with_system(add_bone.before(complete_action))
        .with_system(remove_bone.before(complete_action))
        .with_system(draw_debug_lines.after(complete_action))
        .with_system(complete_action)
        .with_system(select_bone.after(complete_action))
}

pub fn start_action(
    cursor_pos: Res<CursorPos>,
    mut state: ResMut<State>,
    keys: Res<Input<KeyCode>>,
    q: Query<(&Transform, &Bone, Entity)>,
) {
    // Start action only if action isn't already taken
    if state.action != Action::None {
        return;
    }
    if keys.just_pressed(KeyCode::G) {
        state.action = Action::Translate;
    } else if keys.just_pressed(KeyCode::S) {
        state.action = Action::Scale;
    } else if keys.just_pressed(KeyCode::R) {
        state.action = Action::Rotate;
    } else {
        return;
    }
    // Store cursor position at moment action is started
    state.cursor_anchor = cursor_pos.0;
    // Find selected entities and store their transforms at moment action is started
    state.original_transforms.clear();
    state.selected_entities.clear();
    for (transform, bone, entity) in q.iter() {
        if bone.is_selected {
            state.original_transforms.push(transform.clone());
            state.selected_entities.push(entity);
        }
    }
    // Don't start action if no bones are selected
    if state.selected_entities.len() == 0 {
        state.action = Action::None;
    }
}

pub fn complete_action(mouse: Res<Input<MouseButton>>, mut state: ResMut<State>) {
    // If current action is a transformation finnish this action
    if state.action != Action::None && state.action != Action::Done {
        if mouse.just_pressed(MouseButton::Left) {
            state.action = Action::Done
        }
    // Otherwise set state.action to None in case it was Done
    } else {
        state.action = Action::None;
    }
}

pub fn remove_bone(mut commands: Commands, keys: Res<Input<KeyCode>>, q: Query<(Entity, &Bone)>) {
    // Remove bone only if DELETE was pressed
    if !keys.just_pressed(KeyCode::Delete) {
        return;
    }

    for (entity, bone) in q.iter() {
        if bone.is_selected {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn transform_bone(
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, Option<&Parent>, &mut Transform), With<Bone>>,
    state: Res<State>,
) {
    match state.action {
        Action::Translate => {
            for i in 0..state.selected_entities.len() {
                if let Some(parent) = q.get(state.selected_entities[i]).unwrap().1 {
                    let parent_entity = parent.0;
                    // Calculate transform relative to parent entity
                    let parent_gl_transform = q.get(parent_entity).unwrap().0;
                    let v_diff = cursor_pos.0 - state.cursor_anchor;
                    let v_diff_vec3 = Vec3::new(v_diff.x, v_diff.y, 0.);
                    let rel_translation =
                        Quat::mul_vec3(Quat::inverse(parent_gl_transform.rotation), v_diff_vec3)
                            / Vec3::new(
                                parent_gl_transform.scale.x,
                                parent_gl_transform.scale.y,
                                1.,
                            );
                    q.get_mut(state.selected_entities[i]).unwrap().2.translation =
                        state.original_transforms[i].translation + rel_translation;
                } else {
                    // Entity has no parent
                    let v_diff = cursor_pos.0 - state.cursor_anchor;
                    let v_diff_vec3 = Vec3::new(v_diff.x, v_diff.y, 0.);
                    q.get_mut(state.selected_entities[i]).unwrap().2.translation =
                        state.original_transforms[i].translation + v_diff_vec3;
                }
            }
        }
        Action::Rotate => {
            for i in 0..state.selected_entities.len() {
                // Get bone's global transform, vector from bone cursor anchor
                // and vector from bone to current cursor position
                let gl_transform = q.get(state.selected_entities[i]).unwrap().0;
                let mut v_diff_anchor = state.cursor_anchor - gl_transform.translation.truncate();
                let mut v_diff = cursor_pos.0 - gl_transform.translation.truncate();
                // If either v_diff_anchor or v_diff is null vector assign arbitrary value
                if v_diff_anchor.length() == 0. {
                    v_diff_anchor = Vec2::new(0., 1.);
                }
                if v_diff.length() == 0. {
                    v_diff = Vec2::new(0., 1.);
                }
                // Assign changed rotation to bone's transform
                let mut transform = q.get_mut(state.selected_entities[i]).unwrap().2;
                transform.rotation = state.original_transforms[i].rotation
                    * Quat::from_rotation_arc(
                        v_diff_anchor.normalize().extend(0.),
                        v_diff.normalize().extend(0.),
                    );
            }
        }
        Action::Scale => {
            for i in 0..state.selected_entities.len() {
                // Get bone's global transform, vector from bone cursor anchor
                // and vector from bone to current cursor position
                let gl_transform = q.get(state.selected_entities[i]).unwrap().0;
                let v_diff_anchor = state.cursor_anchor - gl_transform.translation.truncate();
                let v_diff = cursor_pos.0 - gl_transform.translation.truncate();
                let distance_to_anchor = f32::max(0.1, v_diff_anchor.length());
                let distance_to_cursor = f32::max(0.1, v_diff.length());
                let scale_ratio = distance_to_cursor / distance_to_anchor;
                let mut transform = q.get_mut(state.selected_entities[i]).unwrap().2;
                transform.scale = state.original_transforms[i].scale * scale_ratio;
            }
        }
        Action::None => (),
        Action::Done => (),
    }
}

pub fn select_bone(
    mouse: Res<Input<MouseButton>>,
    state: Res<State>,
    keys: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, &mut Bone, Entity)>,
) {
    // Select/Unselect only if action is not already taken and if left mouse was pressed
    if !mouse.just_pressed(MouseButton::Left) || state.action != Action::None {
        return;
    }
    let mut closest_entity: Option<Entity> = None;
    let mut shortest_distance = 999.;
    for (gl_transform, _, entity) in q.iter_mut() {
        let length = gl_transform.scale.y;
        let center = gl_transform.translation
            + Quat::mul_vec3(gl_transform.rotation, Vec3::new(0., length / 3., 0.));
        let distance = Vec2::distance(center.truncate(), cursor_pos.0);
        if distance < length / 2. && distance < shortest_distance {
            closest_entity = Some(entity);
            shortest_distance = distance;
        }
    }
    // Select/Unselect bone
    if let Some(closest) = closest_entity {
        let (_, mut bone, _) = q.get_mut(closest).unwrap();
        bone.is_selected = !bone.is_selected;
    }
    // If Shift is not pressed replace selection, otherwise add to selection
    if !keys.pressed(KeyCode::LShift) {
        for (_, mut bone, entity) in q.iter_mut() {
            match closest_entity {
                Some(closest) => {
                    if entity != closest {
                        bone.is_selected = false;
                    }
                }
                None => {
                    bone.is_selected = false;
                }
            }
        }
    }
}

pub fn add_bone(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, &mut Bone, Entity)>,
    mut state: ResMut<State>,
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
    for (_, bone, entity) in q.iter() {
        if bone.is_selected {
            opt_parent = Some(entity);
            break;
        }
    }
    let entity = if let Some(parent) = opt_parent {
        // Spawn as child of parent
        let mut res = Entity::from_bits(0);
        let (parent_gl_transform, _, _) = q.get(parent).unwrap();
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
                .insert(Bone::new())
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
            .insert(Bone::new())
            .id()
    };
    // Unselect all bones
    for (_, mut bone, _) in q.iter_mut() {
        bone.is_selected = false;
    }
    state.action = Action::Done;
}

pub fn draw_debug_lines(
    mut debug_drawer: ResMut<DebugDrawer>,
    bone_gl_transforms: Query<(&GlobalTransform, &Bone)>,
) {
    for (gl_transform, bone) in bone_gl_transforms.iter() {
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
                if bone.is_selected {
                    COLOR_SELECTED
                } else {
                    COLOR_DEFAULT
                },
            );
        }
    }
}
