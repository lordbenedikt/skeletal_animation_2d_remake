use crate::{bone::Bone, *};
use bevy::{prelude::*, utils::HashMap};

pub struct ShowKeyframeEvent {
    pub animation_name: String,
    pub keyframe_index: usize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BlendingStyle {
    Layering,
    FourWayAdditive,
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
        }
    }
}

pub struct State {
    pub running: bool,
    pub layers: Vec<String>,
    pub blending_style: BlendingStyle,
    start_time: f64,
}
impl State {
    pub fn new() -> State {
        State {
            running: false,
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
    pub keyframe_indices: Vec<usize>,
    pub transforms: Vec<Transform>,
    pub interpolation_functions: Vec<interpolate::Function>,
}
impl ComponentAnimation {
    pub fn remove_keyframe(&mut self, index: usize) {
        for i in (0..self.keyframe_indices.len()).rev() {
            if self.keyframe_indices[i] == index {
                self.keyframe_indices.remove(i);
                self.transforms.remove(i);
                self.interpolation_functions.remove(i);
            } else if self.keyframe_indices[i] > index {
                self.keyframe_indices[i] -= 1;
            }
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(start_stop)
        .with_system(apply_animation)
        .with_system(create_keyframe)
        .with_system(show_keyframe)
}

pub fn start_stop(keys: Res<Input<KeyCode>>, mut state: ResMut<State>, time: Res<Time>) {
    if keys.just_pressed(KeyCode::P) {
        state.running = !state.running;
        state.start_time = time.seconds_since_startup();
    }
}

pub fn apply_animation(
    mut q: Query<(&mut Transform, Option<&Bone>), With<Animatable>>,
    state: ResMut<State>,
    anims: Res<Animations>,
    time: Res<Time>,
) {
    // Only apply if any animation is available and running == true
    if anims.map.is_empty() || state.running == false {
        return;
    }

    if state.blending_style == BlendingStyle::Layering {
        for anim_name in state.layers.iter() {
            let mut anim = anims.map.get(anim_name).unwrap();
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
                if q.get_mut(key).is_err() || comp_animation.keyframe_indices.len() == 0 {
                    continue;
                }
                if let Some(bone) = q.get_mut(key).unwrap().1 {
                    if bone.is_ccd_maneuvered {
                        continue;
                    }
                }

                let mut current_frame_a = 0;
                for i in 0..comp_animation.keyframe_indices.len() {
                    if time_diff > anim.keyframes[comp_animation.keyframe_indices[i]] {
                        current_frame_a = i;
                    }
                }
                let mut current_frame_b =
                    (current_frame_a + 1) % comp_animation.keyframe_indices.len();

                // Calculate keyframe length
                let keyframe_length_in_secs = if current_frame_b == 0 {
                    // if loop is ending, set to 1.
                    1.
                } else {
                    anim.keyframes[comp_animation.keyframe_indices[current_frame_b]]
                        - anim.keyframes[comp_animation.keyframe_indices[current_frame_a]]
                };

                let mut x = if anim_length_in_secs == 0.0 {
                    0.0
                } else {
                    let comp_time_diff = time_diff
                        % anim.keyframes[*comp_animation.keyframe_indices.iter().last().unwrap()];
                    ((comp_time_diff
                        - anim.keyframes[comp_animation.keyframe_indices[current_frame_a]])
                        / keyframe_length_in_secs) as f32
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
                transform.translation = interpolate::lerp(
                    comp_animation.transforms[current_frame_a].translation,
                    comp_animation.transforms[current_frame_b].translation,
                    x,
                );
                transform.rotation = Quat::lerp(
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
    } else if state.blending_style == BlendingStyle::FourWayAdditive {
        for anim_name in state.layers.iter() {
            let mut anim = anims.map.get(anim_name).unwrap();
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
                if q.get_mut(key).is_err() || comp_animation.keyframe_indices.len() == 0 {
                    continue;
                }
                if let Some(bone) = q.get_mut(key).unwrap().1 {
                    if bone.is_ccd_maneuvered {
                        continue;
                    }
                }

                let mut current_frame_a = 0;
                for i in 0..comp_animation.keyframe_indices.len() {
                    if time_diff > anim.keyframes[comp_animation.keyframe_indices[i]] {
                        current_frame_a = i;
                    }
                }
                let mut current_frame_b =
                    (current_frame_a + 1) % comp_animation.keyframe_indices.len();

                // Calculate keyframe length
                let keyframe_length_in_secs = if current_frame_b == 0 {
                    // if loop is ending, set to 1.
                    1.
                } else {
                    anim.keyframes[comp_animation.keyframe_indices[current_frame_b]]
                        - anim.keyframes[comp_animation.keyframe_indices[current_frame_a]]
                };

                let mut x = if anim_length_in_secs == 0.0 {
                    0.0
                } else {
                    let comp_time_diff = time_diff
                        % anim.keyframes[*comp_animation.keyframe_indices.iter().last().unwrap()];
                    ((comp_time_diff
                        - anim.keyframes[comp_animation.keyframe_indices[current_frame_a]])
                        / keyframe_length_in_secs) as f32
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
                transform.translation = interpolate::lerp(
                    comp_animation.transforms[current_frame_a].translation,
                    comp_animation.transforms[current_frame_b].translation,
                    x,
                );
                transform.rotation = Quat::lerp(
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
}

pub fn create_keyframe(
    q: Query<(&Transform, &Transformable, Entity), With<Animatable>>,
    keys: Res<Input<KeyCode>>,
    egui_state: Res<egui::State>,
    mut anims: ResMut<Animations>,
) {
    // Create KeyFrame only if K was pressed
    if !keys.just_pressed(KeyCode::K) {
        return;
    }
    let anim_name = &egui_state.plots[egui_state.edit_animation].name;
    if !anims.map.contains_key(anim_name) {
        anims
            .map
            .insert(anim_name.to_string(), Animation::default());
    }
    let anims_mut = anims.map.get_mut(anim_name).unwrap();

    // Add keyframe
    anims_mut.keyframes.push(if anims_mut.keyframes.len() == 0 {
        0.0
    } else {
        anims_mut.keyframes.iter().last().unwrap() + egui_state.keyframe_length as f64 / 1000.
    });

    for (transform, transformable, entity) in q.iter() {
        // Only add keyframe for selected objects
        if !transformable.is_selected {
            continue;
        }
        if !anims_mut.comp_animations.contains_key(&entity) {
            anims_mut
                .comp_animations
                .insert(entity, ComponentAnimation::default());
        }
        let bone_animation = anims_mut.comp_animations.get_mut(&entity).unwrap();
        bone_animation
            .interpolation_functions
            .push(egui_state.interpolation_function);
        bone_animation
            .keyframe_indices
            .push(anims_mut.keyframes.len() - 1);
        bone_animation.transforms.push(Transform {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        });
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
            for i in 0..comp_animation.keyframe_indices.len() {
                if comp_animation.keyframe_indices[i] == ev.keyframe_index {
                    let mut transform = q.get_mut(entity).unwrap();
                    transform.translation = comp_animation.transforms[i].translation;
                    transform.scale = comp_animation.transforms[i].scale;
                    transform.rotation = comp_animation.transforms[i].rotation;
                }
            }
        }
    }
}
