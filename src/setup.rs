use bevy::{prelude::*, render::camera::{DepthCalculation, ScalingMode, Camera2d}};

#[derive(Component)]
pub struct MainCamera;

pub fn setup(mut commands: Commands) {
    commands.spawn_bundle(new_camera_2d()).insert(MainCamera);
}

fn new_camera_2d() -> OrthographicCameraBundle<Camera2d> {
    let far = 1000.0;
    let mut camera = OrthographicCameraBundle::new_2d();
    camera.orthographic_projection = OrthographicProjection {
        far,
        depth_calculation: DepthCalculation::ZDifference,
        scaling_mode: ScalingMode::FixedHorizontal,
        ..Default::default()
    };
    camera.transform.scale = Vec3::new(10., 10., 1.);
    return camera;
}