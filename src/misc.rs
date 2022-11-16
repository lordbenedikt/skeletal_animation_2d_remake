use std::cmp;
use std::thread::sleep;

use crate::{skin::AVAILABLE_IMAGES, *};
use bevy::{
    ecs::change_detection::MutUntyped,
    prelude::*,
    render::camera::{DepthCalculation, RenderTarget},
    utils::HashMap,
};

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct SelectBox;

// TODO: Implement Parent-Child-System for bones using this component
pub struct ParentBone(Entity);

pub fn setup(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    clear_color: Res<ClearColor>,
    mut save_load_state: ResMut<save_load::State>,
) {
    commands.spawn_bundle(new_camera_2d()).insert(MainCamera);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: bevy_image::ColorUtils::invert(&clear_color.0),
                ..Default::default()
            },
            visibility: Visibility { is_visible: false },
            ..Default::default()
        })
        .insert(SelectBox);

    // On WASM load all images on startup
    #[cfg(target_arch = "wasm32")]
    {
        for image_name in AVAILABLE_IMAGES.iter() {
            let _ = asset_server.load::<Image, &str>(&format!("img/{}", image_name));
        }
    }

    // Load arachnoid animation
    save_load_state.opt_load_path = Some(save_load::anim_name_to_path("arachnoid"));
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_resize_window(mut windows: ResMut<Windows>) {
    let window = web_sys::window().unwrap();
    let w = window.inner_width().unwrap().as_f64().unwrap();
    let h = window.inner_height().unwrap().as_f64().unwrap();
    let window = windows.get_primary_mut().unwrap();
    window.set_resolution(w as f32, h as f32);
}

fn new_camera_2d() -> Camera2dBundle {
    let far = 1000.0;
    let mut camera = Camera2dBundle::default();
    camera.projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        // scaling_mode: ScalingMode::FixedHorizontal,
        scale: 1f32,
        ..Default::default()
    };
    camera.transform.scale = Vec3::new(
        1. / (PIXELS_PER_UNIT as f32 * 0.5),
        1. / (PIXELS_PER_UNIT as f32 * 0.5),
        1.,
    );
    return camera;
}

pub fn get_mouse_position(
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    // resource that stores cursor position
    mut cursor_pos: ResMut<CursorPos>,
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    let (camera, camera_transform) = q_camera.single();

    // get the window that the camera is displaying to (or the primary window)
    let wnd = if let RenderTarget::Window(id) = camera.target {
        wnds.get(id).unwrap()
    } else {
        wnds.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(screen_pos) = wnd.cursor_position() {
        // get the size of the window
        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

        // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
        let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

        // matrix for undoing the projection and camera transform
        let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

        // use it to convert ndc to world-space coordinates
        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

        // reduce it to a 2D value
        let world_pos: Vec2 = world_pos.truncate();

        cursor_pos.0 = Vec2::new(world_pos.x, world_pos.y);
    }
}

pub fn map(value: f32, from: [f32; 2], to: [f32; 2]) -> f32 {
    if from[0] == from[1] || to[0] == to[1] {
        return to[0];
    }
    if value <= from[0] && value <= from[1] {
        if from[0] < from[1] {
            return to[0];
        } else {
            return to[1];
        }
    }
    if value >= from[0] && value >= from[1] {
        if from[0] < from[1] {
            return to[1];
        } else {
            return to[0];
        }
    }
    let progress = (value - from[0]) / (from[1] - from[0]);
    let to_diff = to[1] - to[0];
    to[0] + progress * to_diff
}

pub trait Hash {
    fn hash(&self) -> u64;
}
impl Hash for Vec2 {
    fn hash(&self) -> u64 {
        ((self.x.to_bits() as u64) << 32) + (self.y.to_bits() as u64)
    }
}