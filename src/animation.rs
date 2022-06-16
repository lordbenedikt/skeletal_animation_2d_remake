use crate::*;
use bevy::{prelude::*, utils::HashMap};

pub struct State {
    running: bool,
    start_time: f64,
}
impl State {
    pub fn new() -> State {
        State {
            running: false,
            start_time: 0.,
        }
    }
}

pub struct Animations {
    map: HashMap<String, Animation>,
}
impl Animations {
    pub fn new() -> Animations {
        Animations {
            map: HashMap::new(),
        }
    }
}

#[derive(Default)]
struct Animation {
    // start: f32,
    // end: f32,
    bone_animations: HashMap<Entity, KeyFrames>,
}

#[derive(Default)]
struct KeyFrames {
    vector: Vec<KeyFrame>,
}

struct KeyFrame {
    translation: Vec3,
    rotation: Quat,
    scale: Vec3,
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

pub fn apply_animation(
    mut q: Query<&mut Transform, With<bone::Bone>>,
    mut state: ResMut<State>,
    anims: Res<Animations>,
    time: Res<Time>,
) {
    // Only apply if any animation is available and running == true
    if anims.map.is_empty() || state.running == false {
        return;
    }
    let anim_length_in_secs = 3.;
    let time_diff = time.seconds_since_startup() - state.start_time;
    for (key, value) in &anims.map.get("test").unwrap().bone_animations {
        let current_frame_a = f64::floor(time_diff / anim_length_in_secs) as usize % value.vector.len();
        let current_frame_b = (current_frame_a + 1) % value.vector.len();
        let mut x = ((time_diff % anim_length_in_secs) / anim_length_in_secs) as f32;
        x = interpolate::ease_in_out(x);
        let mut transform = q.get_mut(*key).unwrap();
        transform.translation = interpolate::lerp(
            value.vector[current_frame_a].translation,
            value.vector[current_frame_b].translation,
            x,
        );
        transform.rotation = Quat::lerp(
            value.vector[current_frame_a].rotation,
            value.vector[current_frame_b].rotation,
            x,
        );
        transform.scale = interpolate::lerp(
            value.vector[current_frame_a].scale,
            value.vector[current_frame_b].scale,
            x,
        );
    }
}

pub fn create_key_frame(
    q: Query<(&Transform, Entity), With<bone::Bone>>,
    keys: Res<Input<KeyCode>>,
    mut anims: ResMut<Animations>,
) {
    // Create KeyFrame only if K was pressed
    if !keys.just_pressed(KeyCode::K) {
        return;
    }
    let anim_name = "test".to_string();
    if !anims.map.contains_key(&anim_name) {
        anims
            .map
            .insert(anim_name.to_string(), Animation::default());
    }
    let anims_mut = anims.map.get_mut(&anim_name).unwrap();
    for (transform, entity) in q.iter() {
        if !anims_mut.bone_animations.contains_key(&entity) {
            anims_mut
                .bone_animations
                .insert(entity, KeyFrames::default());
        }
        anims_mut
            .bone_animations
            .get_mut(&entity)
            .unwrap()
            .vector
            .push(KeyFrame {
                translation: transform.translation,
                rotation: transform.rotation,
                scale: transform.scale,
            });
    }
}
