use std::ops::MulAssign;

use crate::{bone::Bone, *};
use bevy::{math, prelude::*, utils::HashMap};
use serde::*;

pub struct ShowKeyframeEvent {
    pub animation_name: String,
    pub keyframe_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum BlendingStyle {
    Layering,
    FourWayAdditive,
    Weights,
}
impl BlendingStyle {
    /// Get a vector containing all interpolation functions
    pub fn all() -> impl ExactSizeIterator<Item = BlendingStyle> {
        [Self::Layering, Self::FourWayAdditive].iter().copied()
    }
}
impl ToString for BlendingStyle {
    fn to_string(&self) -> String {
        match self {
            BlendingStyle::Layering => String::from("layering"),
            BlendingStyle::FourWayAdditive => String::from("4 way additive"),
            BlendingStyle::Weights => String::from("weights"),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct State {
    pub running: bool,
    pub layers: Vec<String>,
    pub blending_style: BlendingStyle,
    start_time: f64,
}
impl State {
    pub fn new() -> State {
        State {
            running: true,
            start_time: 0.,
            layers: vec![String::from("anim_0")],
            blending_style: BlendingStyle::Layering,
        }
    }
}

pub struct Animations {
    pub map: HashMap<String, Animation>,
}
impl Animations {
    pub fn new() -> Animations {
        let mut map = HashMap::new();
        map.insert(String::from("anim_0"), Animation::default());
        Animations { map }
    }
}

#[derive(Component)]
pub struct Animatable;

/** An animation has keyframes and  a component animation
 *  For each component a transform for each keyframe is being saved
 *  There is also an easing function for each component, but this should be changed to one easing function for each keyframe
 *  Each ComponentAnimation mus have exactly the same amount of transforms as there are keyframes in the animation
 */
#[derive(Default)]
pub struct Animation {
    pub keyframes: Vec<f64>,
    pub comp_animations: HashMap<Entity, ComponentAnimation>,
}
impl Animation {
    pub fn remove_keyframe(&mut self, index: usize) {
        if index >= self.keyframes.len() {
            return;
        }
        self.keyframes.remove(index);
        for bone_animation in self.comp_animations.values_mut() {
            bone_animation.remove_keyframe(index);
        }
    }
}

#[derive(Default)]
pub struct ComponentAnimation {
    pub transforms: Vec<Transform>,
    pub interpolation_functions: Vec<interpolate::Function>,
}
impl ComponentAnimation {
    pub fn remove_keyframe(&mut self, index: usize) {
        if self.transforms.len() > index {
            self.transforms.remove(index);
        }
        if self.interpolation_functions.len() > index {
            self.interpolation_functions.remove(index);
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(start_stop)
        .with_system(apply_animation)
        .with_system(create_or_change_keyframe)
        .with_system(show_keyframe)
        .with_system(check_which_animatables_are_part_of_current_layer)
}

pub fn check_which_animatables_are_part_of_current_layer(
    mut q: Query<(Entity, &mut Transformable)>,
    egui_state: Res<egui::State>,
    animations: Res<animation::Animations>,
    mouse: Res<Input<MouseButton>>,
    keys: Res<Input<KeyCode>>,
) {
    if !mouse.just_released(MouseButton::Left) && keys.just_released(KeyCode::K) {
        return;
    }

    let current_animation_name = &egui_state.plots[egui_state.edit_plot].name;
    let current_animation = animations.map.get(current_animation_name);
    if current_animation.is_none() {
        for (_, mut transformable) in q.iter_mut() {
            transformable.is_part_of_layer = false;
        }
    } else {
        for (entity, mut transformable) in q.iter_mut() {
            let anim = current_animation.unwrap();
            let mut found = false;
            for (&e, _) in anim.comp_animations.iter() {
                if e == entity {
                    found = true;
                    break;
                }
            }
            transformable.is_part_of_layer = found;
        }
    }
}

pub fn start_stop(keys: Res<Input<KeyCode>>, mut state: ResMut<State>, time: Res<Time>) {
    if keys.just_pressed(KeyCode::P) {
        state.running = !state.running;
        state.start_time = time.seconds_since_startup();
    }
}

pub fn apply_animation(
    mut q: Query<(&mut Transform, Option<&Bone>), With<Animatable>>,
    cursor_pos: Res<CursorPos>,
    state: ResMut<State>,
    anims: Res<Animations>,
    time: Res<Time>,
    windows: Res<Windows>,
) {
    // Only apply if any animation is available and running == true
    if anims.map.is_empty() || state.running == false {
        return;
    }

    if state.blending_style == BlendingStyle::Layering {
        for anim_name in state.layers.iter() {
            let anim;
            if anims.map.get(anim_name).is_some() {
                anim = anims.map.get(anim_name).unwrap();
            } else {
                continue;
            }
            if anim.keyframes.is_empty() {
                return;
            }
            let anim_length_in_secs = anim.keyframes.iter().last().unwrap() - anim.keyframes[0]; // + 1.;
            let time_diff = (time.seconds_since_startup() - state.start_time) % anim_length_in_secs;
            for (&key, comp_animation) in anim.comp_animations.iter() {
                if q.get_mut(key).is_err() || comp_animation.transforms.len() == 0 {
                    continue;
                }
                if let Some(bone) = q.get_mut(key).unwrap().1 {
                    if bone.is_ccd_maneuvered {
                        continue;
                    }
                }

                let mut current_frame_a = 0;
                for i in 0..comp_animation.transforms.len() {
                    if time_diff > anim.keyframes[i] {
                        current_frame_a = i;
                    }
                }
                let mut current_frame_b = (current_frame_a + 1) % comp_animation.transforms.len();

                // Calculate keyframe length
                let keyframe_length_in_secs = if current_frame_b == 0 {
                    // if loop is ending, set to 1.
                    1.
                } else {
                    anim.keyframes[current_frame_b] - anim.keyframes[current_frame_a]
                };

                let mut x = if anim_length_in_secs == 0.0 {
                    0.0
                } else {
                    let comp_time_diff = time_diff % anim.keyframes.last().unwrap();
                    ((comp_time_diff - anim.keyframes[current_frame_a]) / keyframe_length_in_secs)
                        as f32
                };
                x = match comp_animation.interpolation_functions[current_frame_b] {
                    interpolate::Function::Linear => x,
                    interpolate::Function::EaseInOut => interpolate::ease_in_out(x),
                    interpolate::Function::EaseIn => interpolate::ease_in(x),
                    interpolate::Function::EaseOut => interpolate::ease_out(x),
                    interpolate::Function::EaseOutElastic => interpolate::ease_out_elastic(x),
                    interpolate::Function::EaseInOutElastic => interpolate::ease_in_out_elastic(x),
                    interpolate::Function::EaseInOutBack => interpolate::ease_in_out_back(x),
                };
                let (mut transform, _) = q.get_mut(key).unwrap();

                if transform_is_valid(&transform) {
                    transform.translation = interpolate::lerp(
                        comp_animation.transforms[current_frame_a].translation,
                        comp_animation.transforms[current_frame_b].translation,
                        x,
                    );
                    transform.rotation = quat_nlerp(
                        comp_animation.transforms[current_frame_a].rotation,
                        comp_animation.transforms[current_frame_b].rotation,
                        x,
                    );
                    transform.scale = interpolate::lerp(
                        comp_animation.transforms[current_frame_a].scale,
                        comp_animation.transforms[current_frame_b].scale,
                        x,
                    );
                }
            }
        }
    } else if state.blending_style == BlendingStyle::FourWayAdditive {
        if state.layers.len() < 4 {
            return;
        }

        let mouse_pos = cursor_pos.0;
        let window_height = windows.get_primary().unwrap().height();
        let distance = window_height / PIXELS_PER_UNIT as f32;
        let max_distance = distance * std::f32::consts::SQRT_2;

        let mut up_weight = mouse_pos.distance(Vec2::new(0., distance)) / max_distance;
        up_weight = 1. - up_weight.min(1.).max(0.);
        let mut down_weight = mouse_pos.distance(Vec2::new(0., -distance)) / max_distance;
        down_weight = 1. - down_weight.min(1.).max(0.);
        let mut left_weight = mouse_pos.distance(Vec2::new(-distance, 0.)) / max_distance;
        left_weight = 1. - left_weight.min(1.).max(0.);
        let mut right_weight = mouse_pos.distance(Vec2::new(distance, 0.)) / max_distance;
        right_weight = 1. - right_weight.min(1.).max(0.);

        let total_weight = up_weight + down_weight + left_weight + right_weight;
        if total_weight != 0. {
            up_weight /= total_weight;
            down_weight /= total_weight;
            left_weight /= total_weight;
            right_weight /= total_weight;
        } else {
            up_weight = 0.25;
            down_weight = 0.25;
            left_weight = 0.25;
            right_weight = 0.25;
        }

        // Blend Four Animations
        let mut first = true;
        for i in 0..4 {
            let mut weight = 0.;

            // choose correct weight
            if i == 0 {
                weight = up_weight;
            } else if i == 1 {
                weight = down_weight;
            } else if i == 2 {
                weight = left_weight;
            } else {
                weight = right_weight;
            }

            let mut anim;
            if anims.map.get(&state.layers[i]).is_some() {
                anim = anims.map.get(&state.layers[i]).unwrap();
            } else {
                continue;
            }
            if anim.keyframes.is_empty() {
                return;
            }
            let anim_length_in_secs = anim.keyframes.iter().last().unwrap() - anim.keyframes[0]; // + 1.;
            let time_diff = (time.seconds_since_startup() - state.start_time) % anim_length_in_secs;
            for (&key, comp_animation) in anim.comp_animations.iter() {
                if q.get_mut(key).is_err() || comp_animation.transforms.len() == 0 {
                    continue;
                }
                if let Some(bone) = q.get_mut(key).unwrap().1 {
                    if bone.is_ccd_maneuvered {
                        continue;
                    }
                }

                let mut current_frame_a = 0;
                for i in 0..comp_animation.transforms.len() {
                    if time_diff > anim.keyframes[i] {
                        current_frame_a = i;
                    }
                }
                let mut current_frame_b = (current_frame_a + 1) % comp_animation.transforms.len();

                // Calculate keyframe length
                let keyframe_length_in_secs = if current_frame_b == 0 {
                    // if loop is ending, set to 1.
                    1.
                } else {
                    anim.keyframes[current_frame_b] - anim.keyframes[current_frame_a]
                };

                let mut x = if anim_length_in_secs == 0.0 {
                    0.0
                } else {
                    let comp_time_diff = time_diff % anim.keyframes.last().unwrap();
                    ((comp_time_diff - anim.keyframes[current_frame_a]) / keyframe_length_in_secs)
                        as f32
                };
                x = match comp_animation.interpolation_functions[current_frame_b] {
                    interpolate::Function::Linear => x,
                    interpolate::Function::EaseInOut => interpolate::ease_in_out(x),
                    interpolate::Function::EaseIn => interpolate::ease_in(x),
                    interpolate::Function::EaseOut => interpolate::ease_out(x),
                    interpolate::Function::EaseOutElastic => interpolate::ease_out_elastic(x),
                    interpolate::Function::EaseInOutElastic => interpolate::ease_in_out_elastic(x),
                    interpolate::Function::EaseInOutBack => interpolate::ease_in_out_back(x),
                };
                let (mut transform, _) = q.get_mut(key).unwrap();

                if first {
                    transform.translation = Vec3::new(0., 0., 0.);
                    transform.rotation = Quat::IDENTITY;
                    transform.scale = Vec3::new(0., 0., 0.);
                }

                transform.translation += interpolate::lerp(
                    comp_animation.transforms[current_frame_a].translation,
                    comp_animation.transforms[current_frame_b].translation,
                    x,
                ) * weight;
                transform.rotation *= Quat::lerp(
                    comp_animation.transforms[current_frame_a].rotation,
                    comp_animation.transforms[current_frame_b].rotation,
                    x,
                )
                .lerp(Quat::IDENTITY, 1. - weight);
                transform.scale += interpolate::lerp(
                    comp_animation.transforms[current_frame_a].scale,
                    comp_animation.transforms[current_frame_b].scale,
                    x,
                ) * weight;
            }

            first = false;
        }
    }
}

pub fn create_or_change_keyframe(
    q: Query<(&Transform, &Transformable, Entity), With<Animatable>>,
    keys: Res<Input<KeyCode>>,
    egui_state: Res<egui::State>,
    mut anims: ResMut<Animations>,
) {
    // Create KeyFrame only if K was pressed, edit selected keyframe if J was pressed
    let is_create = keys.just_pressed(KeyCode::K);
    let is_change = keys.just_pressed(KeyCode::J);
    if !is_create && !is_change {
        return;
    }

    let anim_name = &egui_state.plots[egui_state.edit_plot].name;
    if !anims.map.contains_key(anim_name) {
        anims
            .map
            .insert(anim_name.to_string(), Animation::default());
    }
    let anim_mut = anims.map.get_mut(anim_name).unwrap();

    if is_create {
        // Add keyframe
        anim_mut.keyframes.push(if anim_mut.keyframes.len() == 0 {
            0.0
        } else {
            anim_mut.keyframes.iter().last().unwrap() + egui_state.keyframe_length as f64 / 1000.
        });
    }

    for (transform, transformable, entity) in q.iter() {
        // Only add keyframe for selected objects, or ones that are already part of animation
        if !transformable.is_selected && !transformable.is_part_of_layer {
            continue;
        }
        if !anim_mut.comp_animations.contains_key(&entity) {
            anim_mut
                .comp_animations
                .insert(entity, ComponentAnimation::default());
        }
        let comp_animation = anim_mut.comp_animations.get_mut(&entity).unwrap();

        // Fill any missing keyframes with current pos
        while comp_animation.transforms.len() < anim_mut.keyframes.len() {
            comp_animation.transforms.push(transform.clone());
            comp_animation
                .interpolation_functions
                .push(egui_state.interpolation_function);
        }
        if is_change {
            let index = egui_state.plots[egui_state.edit_plot].selected_keyframe_index;
            if anim_mut.keyframes.len() > index {
                if comp_animation.transforms.get_mut(index).is_none() {
                    continue;
                }
                *comp_animation.transforms.get_mut(index).unwrap() = transform.clone();
            }
        }
    }
}

pub fn show_keyframe(
    mut show_keyframe_evr: EventReader<ShowKeyframeEvent>,
    mut q: Query<&mut Transform>,
    state: Res<State>,
    animations: ResMut<Animations>,
) {
    if state.running {
        return;
    }
    for ev in show_keyframe_evr.iter() {
        // Set Transforms to values stored in keyframe
        for (&entity, comp_animation) in animations
            .map
            .get(&ev.animation_name)
            .unwrap()
            .comp_animations
            .iter()
        {
            for i in 0..comp_animation.transforms.len() {
                if i == ev.keyframe_index {
                    let mut transform = q.get_mut(entity).unwrap();
                    transform.translation = comp_animation.transforms[i].translation;
                    transform.scale = comp_animation.transforms[i].scale;
                    transform.rotation = comp_animation.transforms[i].rotation;
                }
            }
        }
    }
}

fn transform_is_valid(transform: &Transform) -> bool {
    let mut invalid =
        transform.translation.is_nan() || transform.scale.is_nan() || transform.rotation.is_nan();
    invalid |= transform.scale.x == 0.0 || transform.scale.y == 0.0 || transform.scale.z == 0.0;
    !invalid
}

//// Normalised Lerp function for quaternions
//// Taken from: https://stackoverflow.com/questions/46156903/how-to-lerp-between-two-quaternions
fn quat_dot(a: Quat, b: Quat) -> f32 {
    a.x * b.x + a.y * b.y + a.z * b.z + a.w * b.w
}

fn quat_negate(a: Quat) -> Quat {
    Quat::from_xyzw(-a.x,-a.y,-a.z,-a.w)
}

fn quat_normalise(a: Quat) -> Quat {
    let scalar = 1.0 / f32::sqrt(quat_dot(a,a));
    Quat::from_xyzw(a.x*scalar,a.y*scalar,a.z*scalar,a.w*scalar)
}

fn quat_lerp(a: Quat, b: Quat, x: f32) -> Quat {
    let mut _b = b.clone();

    if quat_dot(a,b) < 0.0 {
        _b = quat_negate(b);
    }

    let cx = a.x - x * (a.x - _b.x);
    let cy = a.y - x * (a.y - _b.y);
    let cz = a.z - x * (a.z - _b.z);
    let cw = a.w - x * (a.w - _b.w);
    Quat::from_xyzw(cx, cy, cz, cw)
}

fn quat_nlerp(a:Quat,b:Quat,x:f32) -> Quat {
    quat_normalise(quat_lerp(a,b,x))
}