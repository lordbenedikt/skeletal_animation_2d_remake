mod animation;
mod bone;
mod debug;
mod interpolate;
mod skin;
mod misc;
mod skeleton;
mod transform;
mod state;

use transform::*;
use bevy::{
    prelude::*,
    render::mesh::*,
    sprite::Mesh2dHandle,
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use debug::DebugDrawer;

const COLOR_SELECTED: Color = Color::rgb(1., 1., 1.);
const COLOR_DEFAULT: Color = Color::rgb(1., 0.6, 0.);

// RESOURCES
pub struct CursorPos(Vec2);

// struct Meshes(Vec<Entity>);
struct Vertices(Vec<Vec2>);

fn main() {
    App::new()
        // RESOURCES
        .insert_resource(WindowDescriptor {
            title: "Skeletal Animation".to_string(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(CursorPos(Vec2::new(0., 0.)))
        .insert_resource(state::State::new())
        .insert_resource(animation::Animations::new())
        .insert_resource(animation::State::new())
        .insert_resource(DebugDrawer::default())
        .insert_resource(skin::Skins::default())
        .insert_resource(skeleton::Skeleton::default())
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        // STARTUP SYSTEMS
        .add_startup_system(misc::setup)
        .add_startup_system(skeleton::create_mesh)
        // SYSTEMS
        // .add_system(add_vertex)
        .add_system(misc::get_mouse_position.label("input_handling"))
        .add_system(skeleton::update_mesh)
        .add_system_set(bone::system_set().label("bone_systems"))
        .add_system_set(transform::system_set().label("tramsform_systems").after("bone_systems"))
        .add_system_set(debug::system_set().after("transform_systems"))
        .add_system_set(animation::system_set())
        // RUN
        .run();
}