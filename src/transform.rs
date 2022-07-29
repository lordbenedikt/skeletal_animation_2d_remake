use crate::{bone::Bone, *};

pub struct State {
    pub action: transform::Action,
    pub cursor_anchor: Vec2,
    pub original_transforms: Vec<Transform>,
    pub selected_entities: Vec<Entity>,
}
impl State {
    pub fn new() -> State {
        State {
            action: transform::Action::None,
            cursor_anchor: Vec2::new(0., 0.),
            original_transforms: vec![],
            selected_entities: vec![],
        }
    }
}

#[derive(Default)]
pub struct UpdateSelectionEvent;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Action {
    None,
    Translate,
    Rotate,
    Scale,
    ScaleX,
    ScaleY,
    Done,
}

#[derive(Component)]
pub struct Transformable {
    pub is_selected: bool,
    pub translatable: bool,
    pub rotatable: bool,
    pub scalable: bool,
    pub collision_shape: Shape,
}
impl Default for Transformable {
    fn default() -> Self {
        Self {
            is_selected: true,
            translatable: true,
            rotatable: true,
            scalable: true,
            collision_shape: Shape::None,
        }
    }
}
impl Transformable {
    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.collision_shape = shape;
        self
    }
}

#[derive(PartialEq)]
pub enum Shape {
    Rectangle(Vec2, Vec2),
    Line(Vec2, Vec2),
    None,
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(update_selection)
        .with_system(start_action)
        .with_system(transform)
        .with_system(remove.before(complete_action))
        .with_system(select.before(complete_action))
        .with_system(complete_action)
}

pub fn start_action(
    cursor_pos: Res<CursorPos>,
    mut state: ResMut<State>,
    keys: Res<Input<KeyCode>>,
    q: Query<&Transform, With<Transform>>,
) {
    // // WIP
    // Switch between scale modi
    // if keys.just_pressed(KeyCode::S) {
    //     if state.action == Action::Scale {
    //         state.action = Action::ScaleX;
    //         return;
    //     }
    //     if state.action == Action::ScaleX {
    //         state.action = Action::ScaleY;
    //         return;
    //     }
    //     if state.action == Action::ScaleY {
    //         state.action = Action::Scale;
    //         return;
    //     }
    // }
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
    // Store selected entities' transforms at moment action is started
    state.original_transforms.clear();
    for e in state.selected_entities.clone() {
        let transform = q.get(e).unwrap();
        state.original_transforms.push(transform.clone());
    }
    // Don't start action if no transformables are selected
    if state.selected_entities.len() == 0 {
        state.action = Action::None;
    }
}

pub fn transform(
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, Option<&Parent>, &mut Transform), With<Transformable>>,
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
                // Get transformable's global transform, vector from transformable to cursor anchor
                // and vector from transformable to current cursor position
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
                // Assign changed rotation to transformable's transform
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
                // Get transformable's global transform, vector from transformable cursor anchor
                // and vector from transformable to current cursor position
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
        Action::ScaleX => {
            for i in 0..state.selected_entities.len() {
                // Get transformable's global transform, vector from transformable cursor anchor
                // and vector from transformable to current cursor position
                let gl_transform = q.get(state.selected_entities[i]).unwrap().0;
                let v_diff_anchor = state.cursor_anchor - gl_transform.translation.truncate();
                let v_diff = cursor_pos.0 - gl_transform.translation.truncate();
                let distance_to_anchor = f32::max(0.1, v_diff_anchor.length());
                let distance_to_cursor = f32::max(0.1, v_diff.length());
                let scale_ratio = distance_to_cursor / distance_to_anchor;
                let mut transform = q.get_mut(state.selected_entities[i]).unwrap().2;
                transform.scale.x = state.original_transforms[i].scale.x * scale_ratio;
            }
        }
        Action::ScaleY => {
            for i in 0..state.selected_entities.len() {
                // Get transformable's global transform, vector from transformable cursor anchor
                // and vector from transformable to current cursor position
                let gl_transform = q.get(state.selected_entities[i]).unwrap().0;
                let v_diff_anchor = state.cursor_anchor - gl_transform.translation.truncate();
                let v_diff = cursor_pos.0 - gl_transform.translation.truncate();
                let distance_to_anchor = f32::max(0.1, v_diff_anchor.length());
                let distance_to_cursor = f32::max(0.1, v_diff.length());
                let scale_ratio = distance_to_cursor / distance_to_anchor;
                let mut transform = q.get_mut(state.selected_entities[i]).unwrap().2;
                transform.scale.y = state.original_transforms[i].scale.y * scale_ratio;
            }
        }
        Action::None => (),
        Action::Done => (),
    }
}

pub fn remove(
    mut commands: Commands,
    keys: Res<Input<KeyCode>>,
    q: Query<(Entity, &Transformable, Option<&skin::Skin>, Option<&Bone>)>,
    mut state: ResMut<State>,
    mut skeleton: ResMut<skeleton::Skeleton>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // Remove transformable only if DELETE was pressed
    if !keys.just_pressed(KeyCode::Delete) {
        return;
    }
    for (entity, transformable, skin, bone) in q.iter() {
        if transformable.is_selected {
            commands.entity(entity).despawn_recursive();
            state.selected_entities.retain(|&e| e != entity);
            if let Some(skin) = skin {
                meshes.remove(skin.mesh_handle.clone().unwrap().0);
            }
        }
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

pub fn select(
    mouse: Res<Input<MouseButton>>,
    mut state: ResMut<State>,
    keys: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, &mut Transformable, Entity)>,
    mut update_selection_evw: EventWriter<UpdateSelectionEvent>,
) {
    // Select/Unselect only if action is not already taken and if left mouse was pressed
    if !mouse.just_pressed(MouseButton::Left) || state.action != Action::None {
        return;
    }

    let mut closest_entity: Option<Entity> = None;
    let mut shortest_distance = 999.;
    for (gl_transform, transformable, entity) in q.iter_mut() {
        let distance: f32 = if let Shape::Rectangle(min, max) = transformable.collision_shape {
            Vec2::distance((min + max) / 2., cursor_pos.0)
        } else if let Shape::Line(start, end) = transformable.collision_shape {
            distance_segment_point(start, end, cursor_pos.0)
        } else {
            // assert transformable.collision_shape == Shape::None
            let length = gl_transform.scale.y;
            let center = gl_transform.translation
                + Quat::mul_vec3(gl_transform.rotation, Vec3::new(0., length / 3., 0.));
            Vec2::distance(center.truncate(), cursor_pos.0)
        };
        if distance < shortest_distance {
            if let Shape::Rectangle(min, max) = transformable.collision_shape {
                if min.x <= cursor_pos.0.x
                    && min.y <= cursor_pos.0.y
                    && max.x >= cursor_pos.0.x
                    && max.y >= cursor_pos.0.y
                {
                    closest_entity = Some(entity);
                    shortest_distance = distance;
                }
            } else if let Shape::Line(start, end) = transformable.collision_shape {
                if distance < 1. {
                    closest_entity = Some(entity);
                    shortest_distance = distance;
                }
            } else {
                if distance < 1. {
                    closest_entity = Some(entity);
                    shortest_distance = distance;
                }
            }
        }
    }
    // Select/Unselect transformable
    if let Some(closest) = closest_entity {
        let (_, mut transformable, _) = q.get_mut(closest).unwrap();
        transformable.is_selected = !transformable.is_selected;
    }
    // If Shift is not pressed replace selection, otherwise add to selection
    if !keys.pressed(KeyCode::LShift) {
        for (_, mut transformable, entity) in q.iter_mut() {
            match closest_entity {
                Some(closest) => {
                    if entity != closest {
                        transformable.is_selected = false;
                    }
                }
                None => {
                    transformable.is_selected = false;
                }
            }
        }
    }
    update_selection_evw.send_default();
}

pub fn get_relative_transform(
    origin: &GlobalTransform,
    gl_transform: &GlobalTransform,
) -> Transform {
    let mut result = gl_transform.clone();
    result.translation -= origin.translation;
    let origin_rotation_inverse = origin.rotation.inverse();
    result.translation = Quat::mul_vec3(origin_rotation_inverse, result.translation);
    result.rotation *= origin_rotation_inverse;
    if origin.scale.x != 0. && origin.scale.y != 0. && origin.scale.z != 0. {
        result.translation /= origin.scale;
        result.scale /= origin.scale;
    } else {
        println!("get_relative_transform: Failed to compute relative transform, because origin's scale is 0");
    }
    Transform {
        translation: result.translation,
        rotation: result.rotation,
        scale: result.scale,
    }
}

pub fn get_global_transform(
    origin: &GlobalTransform,
    rel_transform: &Transform,
) -> GlobalTransform {
    let mut result = rel_transform.clone();
    result.translation += origin.translation;
    result.rotation *= origin.rotation;
    result.scale *= origin.scale;
    GlobalTransform {
        translation: result.translation,
        rotation: result.rotation,
        scale: result.scale,
    }
}

pub fn distance_segment_point(start: Vec2, end: Vec2, v: Vec2) -> f32 {
    let length = Vec2::distance_squared(start, end);
    if length == 0.0 {
        return Vec2::distance(v, start);
    }
    let t = f32::max(0., f32::min(1., Vec2::dot(v - start, end - start) / length));
    let projection: Vec2 = start + t * (end - start);
    return Vec2::distance(v, projection);
}

pub fn update_selection(
    q: Query<(&Transformable, Entity)>,
    mut state: ResMut<State>,
    mut update_selection_evr: EventReader<UpdateSelectionEvent>,
) {
    for _ in update_selection_evr.iter() {
        state.selected_entities.clear();
        for (transformable, entity) in q.iter() {
            if transformable.is_selected {
                state.selected_entities.push(entity);
            }
        }
    }
}
