use bevy::{
    math::Vec3A,
    utils::{HashMap, HashSet},
};

use crate::{bone::Bone, *};

pub struct State {
    pub action: transform::Action,
    pub cursor_anchor: Vec2,
    pub original_transforms: HashMap<Entity, Transform>,
    pub selected_entities: HashSet<Entity>,
    pub drag_select: bool,
}
impl State {
    pub fn new() -> State {
        State {
            action: transform::Action::None,
            cursor_anchor: Vec2::new(0., 0.),
            original_transforms: HashMap::new(),
            selected_entities: HashSet::new(),
            drag_select: false,
        }
    }
}

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
    pub is_part_of_layer: bool,
    pub translatable: bool,
    pub rotatable: bool,
    pub scalable: bool,
    pub collision_shape: Shape,
}
impl Default for Transformable {
    fn default() -> Self {
        Self {
            is_selected: true,
            is_part_of_layer: false,
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
        .with_system(start_action)
        .with_system(transform)
        .with_system(remove.before(complete_action))
        .with_system(select.before(complete_action))
        .with_system(
            start_stop_drag_select
                .after(transform)
                .after(select)
                .before(complete_action),
        )
        .with_system(complete_action)
}

pub fn start_action(
    cursor_pos: Res<CursorPos>,
    mut state: ResMut<State>,
    egui_state: Res<egui::State>,
    keys: Res<Input<KeyCode>>,
    q: Query<&Transform, With<Transform>>,
) {
    // // // WIP
    // // Currently doesn't work with parent-child-hierarchies
    // // To fix it might be necessary to implement own parent-child-system system
    // Switch between scale modi
    if keys.just_released(KeyCode::S) {
        if state.action == Action::Scale {
            state.action = Action::ScaleX;
            return;
        }
        if state.action == Action::ScaleX {
            state.action = Action::ScaleY;
            return;
        }
        if state.action == Action::ScaleY {
            state.action = Action::Scale;
            return;
        }
    }
    // Start action only if action isn't already taken
    if state.action != Action::None {
        return;
    }

    if keys.just_released(KeyCode::G) {
        state.action = Action::Translate;
    } else if keys.just_released(KeyCode::S) {
        state.action = Action::Scale;
    } else if keys.just_released(KeyCode::R) {
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
        state.original_transforms.insert(e, transform.clone());
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
            for (&entity, &orig_transform) in state.original_transforms.iter() {
                if let Some(parent) = q.get(entity).unwrap().1 {
                    // Calculate transform relative to parent entity
                    let parent_entity = parent.get();
                    let parent_gl_transform = q.get(parent_entity).unwrap().0;
                    let v_diff = cursor_pos.0 - state.cursor_anchor;
                    let v_diff_vec3 = Vec3::new(v_diff.x, v_diff.y, 0.);
                    let (parent_gl_scale, parent_gl_rotation, _) =
                        parent_gl_transform.to_scale_rotation_translation();
                    let rel_translation =
                        Quat::mul_vec3(Quat::inverse(parent_gl_rotation), v_diff_vec3)
                            / Vec3::new(parent_gl_scale.x, parent_gl_scale.y, 1.);
                    q.get_mut(entity).unwrap().2.translation =
                        orig_transform.translation + rel_translation;
                } else {
                    // Entity has no parent
                    let v_diff = cursor_pos.0 - state.cursor_anchor;
                    let v_diff_vec3 = Vec3::new(v_diff.x, v_diff.y, 0.);
                    q.get_mut(entity).unwrap().2.translation =
                        orig_transform.translation + v_diff_vec3;
                }
            }
        }
        Action::Rotate => {
            for (&entity, &orig_transform) in state.original_transforms.iter() {
                // Get transformable's global transform, vector from transformable to cursor anchor
                // and vector from transformable to current cursor position
                let gl_transform = q.get(entity).unwrap().0;
                let mut v_diff_anchor =
                    state.cursor_anchor - gl_transform.affine().translation.truncate();
                let mut v_diff = cursor_pos.0 - gl_transform.affine().translation.truncate();
                // If either v_diff_anchor or v_diff is null vector assign arbitrary value
                if v_diff_anchor.length() == 0. {
                    v_diff_anchor = Vec2::new(0., 1.);
                }
                if v_diff.length() == 0. {
                    v_diff = Vec2::new(0., 1.);
                }
                // Assign changed rotation to transformable's transform
                let mut transform = q.get_mut(entity).unwrap().2;
                transform.rotation = orig_transform.rotation
                    * Quat::from_rotation_arc(
                        v_diff_anchor.normalize().extend(0.),
                        v_diff.normalize().extend(0.),
                    );
            }
        }
        Action::Scale => {
            for (&entity, &orig_transform) in state.original_transforms.iter() {
                // Get transformable's global transform, vector from transformable cursor anchor
                // and vector from transformable to current cursor position
                let gl_transform = q.get(entity).unwrap().0;
                let v_diff_anchor =
                    state.cursor_anchor - gl_transform.affine().translation.truncate();
                let v_diff = cursor_pos.0 - gl_transform.affine().translation.truncate();
                let distance_to_anchor = f32::max(0.1, v_diff_anchor.length());
                let distance_to_cursor = f32::max(0.1, v_diff.length());
                let scale_ratio = distance_to_cursor / distance_to_anchor;
                let mut transform = q.get_mut(entity).unwrap().2;
                transform.scale = orig_transform.scale * scale_ratio;
            }
        }
        Action::ScaleX => {
            for (&entity, &orig_transform) in state.original_transforms.iter() {
                // Get transformable's global transform, vector from transformable cursor anchor
                // and vector from transformable to current cursor position
                let gl_transform = q.get(entity).unwrap().0;
                let v_diff_anchor =
                    state.cursor_anchor - gl_transform.affine().translation.truncate();
                let v_diff = cursor_pos.0 - gl_transform.affine().translation.truncate();
                let distance_to_anchor = f32::max(0.1, v_diff_anchor.length());
                let distance_to_cursor = f32::max(0.1, v_diff.length());
                let scale_ratio = distance_to_cursor / distance_to_anchor;
                let mut transform = q.get_mut(entity).unwrap().2;
                transform.scale.x = orig_transform.scale.x * scale_ratio;
            }
        }
        Action::ScaleY => {
            for (&entity, &orig_transform) in state.original_transforms.iter() {
                // Get transformable's global transform, vector from transformable cursor anchor
                // and vector from transformable to current cursor position
                let gl_transform = q.get(entity).unwrap().0;
                let v_diff_anchor =
                    state.cursor_anchor - gl_transform.affine().translation.truncate();
                let v_diff = cursor_pos.0 - gl_transform.affine().translation.truncate();
                let distance_to_anchor = f32::max(0.1, v_diff_anchor.length());
                let distance_to_cursor = f32::max(0.1, v_diff.length());
                let scale_ratio = distance_to_cursor / distance_to_anchor;
                let mut transform = q.get_mut(entity).unwrap().2;
                transform.scale.y = orig_transform.scale.y * scale_ratio;
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
    if !keys.just_released(KeyCode::Delete) {
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
        if mouse.just_released(MouseButton::Left) {
            state.action = Action::Done
        }
    // Otherwise set state.action to None in case it was Done
    } else {
        state.action = Action::None;
    }
}

pub fn start_stop_drag_select(
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
    mut state: ResMut<State>,
    egui_state: Res<egui::State>,
    cursor_pos: Res<CursorPos>,
) {
    // Stop Drag Select
    if mouse.just_released(MouseButton::Left) {
        state.drag_select = false;
    }

    // Select/Unselect only if action is not already taken, left mouse was pressed and ui_hover is false
    if !mouse.pressed(MouseButton::Left)
        || keys.pressed(KeyCode::LControl)
        || state.action != Action::None
        || state.drag_select
        || egui_state.ui_hover
    {
        return;
    }

    if mouse.just_pressed(MouseButton::Left) {
        state.cursor_anchor = cursor_pos.0;
    }

    if !egui_state.ui_drag
        && cursor_pos.0.distance(state.cursor_anchor) > 10. / PIXELS_PER_UNIT as f32
    {
        state.drag_select = true;
    }
}

pub fn select(
    mouse: Res<Input<MouseButton>>,
    mut state: ResMut<State>,
    egui_state: Res<egui::State>,
    keys: Res<Input<KeyCode>>,
    cursor_pos: Res<CursorPos>,
    mut q: Query<(&GlobalTransform, &mut Transformable, Entity)>,
) {
    // Select/Unselect only if conditions are fulfilled
    if !mouse.just_released(MouseButton::Left)
        || state.action != Action::None
        || (egui_state.ui_hover && !state.drag_select)
    {
        return;
    }

    let mut select_entities: Vec<Entity> = vec![];

    if !state.drag_select {
        let mut shortest_distance = 999.;
        for (gl_transform, transformable, entity) in q.iter_mut() {
            let distance: f32 = if let Shape::Rectangle(min, max) = transformable.collision_shape {
                Vec2::distance((min + max) / 2., cursor_pos.0)
            } else if let Shape::Line(start, end) = transformable.collision_shape {
                distance_segment_point(start, end, cursor_pos.0)
            } else {
                // assert transformable.collision_shape == Shape::None
                let length = gl_transform.to_scale_rotation_translation().0.y;
                let center = gl_transform.affine().translation
                    + Quat::mul_vec3a(
                        gl_transform.to_scale_rotation_translation().1,
                        Vec3A::new(0., length / 3., 0.),
                    );
                Vec2::distance(center.truncate(), cursor_pos.0)
            };
            if distance < shortest_distance {
                if let Shape::Rectangle(min, max) = transformable.collision_shape {
                    if min.x <= cursor_pos.0.x
                        && min.y <= cursor_pos.0.y
                        && max.x >= cursor_pos.0.x
                        && max.y >= cursor_pos.0.y
                    {
                        if !select_entities.is_empty() {
                            select_entities.clear();
                        }
                        select_entities.push(entity);
                        shortest_distance = distance;
                    }
                } else if let Shape::Line(start, end) = transformable.collision_shape {
                    if distance < 1. {
                        if !select_entities.is_empty() {
                            select_entities.clear();
                        }
                        select_entities.push(entity);
                        shortest_distance = distance;
                    }
                } else {
                    if distance < 1. {
                        if !select_entities.is_empty() {
                            select_entities.clear();
                        }
                        select_entities.push(entity);
                        shortest_distance = distance;
                    }
                }
            }
        }
    } else {
        for (gl_transform, transformable, entity) in q.iter_mut() {
            let center: Vec2 = if let Shape::Rectangle(min, max) = transformable.collision_shape {
                (min + max) / 2.
            } else if let Shape::Line(start, end) = transformable.collision_shape {
                (start + end) / 2.
            } else {
                // assert transformable.collision_shape == Shape::None
                let length = gl_transform.to_scale_rotation_translation().0.y;
                let center = Vec3::from(gl_transform.affine().translation)
                    + Quat::mul_vec3(
                        gl_transform.to_scale_rotation_translation().1,
                        Vec3::new(0., length / 3., 0.),
                    );
                center.truncate()
            };

            let is_outside_rect = (center.x < cursor_pos.0.x && center.x < state.cursor_anchor.x)
                || (center.x > cursor_pos.0.x && center.x > state.cursor_anchor.x)
                || (center.y < cursor_pos.0.y && center.y < state.cursor_anchor.y)
                || (center.y > cursor_pos.0.y && center.y > state.cursor_anchor.y);
            if !is_outside_rect {
                select_entities.push(entity);
            }
        }
    }

    // If Shift is not pressed..
    if !keys.pressed(KeyCode::LShift) {
        // ..clear selection first
        for (_, mut transformable, entity) in q.iter_mut() {
            if transformable.is_selected {
                state.selected_entities.remove(&entity);
                transformable.is_selected = false;
            }
        }
    }

    if state.drag_select {
        // Add to Selection
        for &e in select_entities.iter() {
            let mut transformable = q.get_mut(e).unwrap().1;
            if !transformable.is_selected {
                state.selected_entities.insert(e);
                transformable.is_selected = true;
            }
        }
    } else {
        // Select/Unselect transformable
        for &e in select_entities.iter() {
            let mut transformable = q.get_mut(e).unwrap().1;
            if transformable.is_selected {
                state.selected_entities.remove(&e);
                transformable.is_selected = false;
            } else {
                state.selected_entities.insert(e);
                transformable.is_selected = true;
            }
        }
    }
}

pub fn get_relative_transform(origin: &GlobalTransform, gl_transform: &Transform) -> Transform {
    let mut result = gl_transform.clone();
    let (origin_scale, origin_rotation, origin_translation) =
        origin.to_scale_rotation_translation();
    result.translation -= origin_translation;
    let origin_rotation_inverse = origin_rotation.inverse();
    result.translation = Quat::mul_vec3(origin_rotation_inverse, result.translation);
    result.rotation *= origin_rotation_inverse;
    if origin_scale.x != 0. && origin_scale.y != 0. && origin_scale.z != 0. {
        result.translation /= origin_scale;
        result.scale /= origin_scale;
    } else {
        println!("get_relative_transform: Failed to compute relative transform, because origin's scale is 0");
    }
    Transform {
        translation: result.translation,
        rotation: result.rotation,
        scale: result.scale,
    }
}

pub fn get_global_transform(origin: &GlobalTransform, rel_transform: &Transform) -> Transform {
    let mut result = rel_transform.clone();
    let (origin_scale, origin_rotation, origin_translation) =
        origin.to_scale_rotation_translation();
    result.translation += origin_translation;
    result.rotation *= origin_rotation;
    result.scale *= origin_scale;
    Transform {
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

pub fn combined_transform(parent: Transform, child: Transform) -> Transform {
    Transform {
        translation: parent.translation
            + parent.rotation.mul_vec3(child.translation * parent.scale),
        rotation: parent.rotation * child.rotation,
        scale: parent.scale * child.scale,
    }
}
