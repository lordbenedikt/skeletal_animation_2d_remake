use crate::{*, bone::Bone};
use bevy::{prelude::*, utils::HashMap};

pub struct State {
    pub running: bool,
    start_time: f64,
    running_animations: Vec<LayeredAnimation>,
}
impl State {
    pub fn new() -> State {
        State {
            running: false,
            start_time: 0.,
            running_animations: Vec::new(),
        }
    }
}

pub struct LayeredAnimation {
    start_time: f64,
    duration: f64,
    animations: Vec<String>,
}

pub struct Animations {
    pub map: HashMap<String, Animation>,
}
impl Animations {
    pub fn new() -> Animations {
        Animations {
            map: HashMap::new(),
        }
    }
}

#[derive(Component)]
pub struct Animatable;

#[derive(Default)]
pub struct Animation {
    pub keyframes: Vec<f64>,
    bone_animations: HashMap<Entity, ComponentAnimation>,
}
impl Animation {
    pub fn remove_keyframe(&mut self, index: usize) {
        if index >= self.keyframes.len() {
            return;
        }
        self.keyframes.remove(index);
        for bone_animation in self.bone_animations.values_mut() {
            bone_animation.remove_keyframe(index);
        }
    }
}

#[derive(Default)]
struct ComponentAnimation {
    keyframe_indices: Vec<usize>,
    transforms: Vec<Transform>,
    interpolation_functions: Vec<interpolate::Function>,
}
impl ComponentAnimation {
    pub fn remove_keyframe(&mut self, index: usize) {
        for i in 0..self.keyframe_indices.len() {
            if self.keyframe_indices[i] == index {
                self.keyframe_indices.remove(i);
                self.transforms.remove(i);
                self.interpolation_functions.remove(i);
            }
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(start_stop)
        .with_system(apply_animation)
        .with_system(create_key_frame)
}

pub fn start_stop(keys: Res<Input<KeyCode>>, mut state: ResMut<State>, time: Res<Time>) {
    if keys.just_pressed(KeyCode::P) {
        state.running = !state.running;
        state.start_time = time.seconds_since_startup();
    }
}

// pub fn apply_animation_old(
//     mut q: Query<&mut Transform, With<bone::Bone>>,
//     state: ResMut<State>,
//     egui_state: Res<egui::State>,
//     anims: Res<Animations>,
//     time: Res<Time>,
// ) {
//     // Only apply if any animation is available and running == true
//     if anims.map.is_empty() || state.running == false {
//         return;
//     }
//     let anim_length_in_secs = egui_state.keyframe_length as f64 / 1000.;
//     let time_diff = time.seconds_since_startup() - state.start_time;
//     for (key, bone_animation) in &anims
//         .map
//         .get(&egui_state.current_animation)
//         .unwrap()
//         .bone_animations
//     {
//         if q.get_mut(*key).is_err() {
//             continue;
//         }

//         let current_frame_a =
//             f64::floor(time_diff / anim_length_in_secs) as usize % bone_animation.transforms.len();
//         let current_frame_b = (current_frame_a + 1) % bone_animation.transforms.len();
//         let mut x = ((time_diff % anim_length_in_secs) / anim_length_in_secs) as f32;
//         x = match egui_state.interpolation_function {
//             interpolate::Function::Linear => x,
//             interpolate::Function::EaseInOut => interpolate::ease_in_out(x),
//             interpolate::Function::EaseIn => interpolate::ease_in(x),
//             interpolate::Function::EaseOut => interpolate::ease_out(x),
//             interpolate::Function::EaseOutElastic => interpolate::ease_out_elastic(x),
//             interpolate::Function::EaseInOutElastic => interpolate::ease_in_out_elastic(x),
//             interpolate::Function::EaseInOutBack => interpolate::ease_in_out_back(x),
//         };
//         let mut transform = q.get_mut(*key).unwrap();
//         transform.translation = interpolate::lerp(
//             bone_animation.transforms[current_frame_a].translation,
//             bone_animation.transforms[current_frame_b].translation,
//             x,
//         );
//         transform.rotation = Quat::lerp(
//             bone_animation.transforms[current_frame_a].rotation,
//             bone_animation.transforms[current_frame_b].rotation,
//             x,
//         );
//         transform.scale = interpolate::lerp(
//             bone_animation.transforms[current_frame_a].scale,
//             bone_animation.transforms[current_frame_b].scale,
//             x,
//         );
//     }
// }

pub fn apply_animation(
    mut q: Query<(&mut Transform, Option<&Bone>), With<Animatable>>,
    state: ResMut<State>,
    egui_state: Res<egui::State>,
    anims: Res<Animations>,
    time: Res<Time>,
) {
    // Only apply if any animation is available and running == true
    if anims.map.is_empty() || state.running == false {
        return;
    }
    // let anim_length_in_secs = egui_state.keyframe_length as f64 / 1000.;
    let anim = anims.map.get(&egui_state.animation.name).unwrap();
    // if no keyframes exist, return
    if anim.keyframes.is_empty() {
        return;
    }
    let anim_length_in_secs = anim.keyframes.iter().last().unwrap() - anim.keyframes[0];
    let time_diff = (time.seconds_since_startup() - state.start_time) % anim_length_in_secs;
    for (key, bone_animation) in &anims
        .map
        .get(&egui_state.animation.name)
        .unwrap()
        .bone_animations
    {
        if q.get_mut(*key).is_err()  || bone_animation.keyframe_indices.len()==0 {
            continue;
        }
        if let Some(bone) = q.get_mut(*key).unwrap().1 {
            if bone.is_ccd_maneuvered {
                continue;
            }
        }

        let mut current_frame_a = 0;
        for i in 0..bone_animation.keyframe_indices.len() {
            if time_diff > anim.keyframes[bone_animation.keyframe_indices[i]] {
                current_frame_a = i;
            }
        }
        let current_frame_b = (current_frame_a + 1) % bone_animation.keyframe_indices.len();

        let keyframe_length_in_secs =
            anim.keyframes[current_frame_b] - anim.keyframes[current_frame_a];
        let mut x = if anim_length_in_secs == 0.0 {
            0.0
        } else {
            ((time_diff - anim.keyframes[current_frame_a]) / keyframe_length_in_secs) as f32
        };
        x = match bone_animation.interpolation_functions[current_frame_b] {
            interpolate::Function::Linear => x,
            interpolate::Function::EaseInOut => interpolate::ease_in_out(x),
            interpolate::Function::EaseIn => interpolate::ease_in(x),
            interpolate::Function::EaseOut => interpolate::ease_out(x),
            interpolate::Function::EaseOutElastic => interpolate::ease_out_elastic(x),
            interpolate::Function::EaseInOutElastic => interpolate::ease_in_out_elastic(x),
            interpolate::Function::EaseInOutBack => interpolate::ease_in_out_back(x),
        };
        let (mut transform, _) = q.get_mut(*key).unwrap();
        transform.translation = interpolate::lerp(
            bone_animation.transforms[current_frame_a].translation,
            bone_animation.transforms[current_frame_b].translation,
            x,
        );
        transform.rotation = Quat::lerp(
            bone_animation.transforms[current_frame_a].rotation,
            bone_animation.transforms[current_frame_b].rotation,
            x,
        );
        transform.scale = interpolate::lerp(
            bone_animation.transforms[current_frame_a].scale,
            bone_animation.transforms[current_frame_b].scale,
            x,
        );
    }
}

pub fn create_key_frame(
    q: Query<(&Transform, Entity), With<Animatable>>,
    keys: Res<Input<KeyCode>>,
    egui_state: Res<egui::State>,
    mut anims: ResMut<Animations>,
) {
    // Create KeyFrame only if K was pressed
    if !keys.just_pressed(KeyCode::K) {
        return;
    }
    let anim_name = &egui_state.animation.name;
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
        anims_mut.keyframes.iter().last().unwrap() + egui_state.animation.keyframe_length as f64 / 1000.
    });

    for (transform, entity) in q.iter() {
        if !anims_mut.bone_animations.contains_key(&entity) {
            anims_mut
                .bone_animations
                .insert(entity, ComponentAnimation::default());
        }
        let bone_animation = anims_mut.bone_animations.get_mut(&entity).unwrap();
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
