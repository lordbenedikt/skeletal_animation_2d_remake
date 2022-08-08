mod animation;
mod bone;
mod cloth;
mod debug;
mod interpolate;
mod mesh;
mod misc;
mod skeleton;
mod skin;
mod transform;
mod egui;
mod ccd;

use bevy::{prelude::*, render::mesh::*, sprite::Mesh2dHandle};
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use bevy_egui::EguiPlugin;
use debug::DebugDrawer;
use transform::*;

const COLOR_WHITE: Color = Color::rgb(1.,1.,1.);
const COLOR_BLACK: Color = Color::rgb(0.,0.,0.);
const COLOR_SELECTED: Color = Color::rgb(1., 0.9, 0.);
const COLOR_DEFAULT: Color = Color::rgb(1., 0.6, 0.);
const PIXELS_PER_UNIT: u32 = 100;

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
        .insert_resource(ClearColor(COLOR_WHITE))
        .insert_resource(CursorPos(Vec2::new(0., 0.)))
        .insert_resource(transform::State::new())
        .insert_resource(animation::Animations::new())
        .insert_resource(animation::State::new())
        .insert_resource(DebugDrawer::default())
        .insert_resource(skin::Skins::default())
        .insert_resource(skeleton::Skeleton::default())
        .insert_resource(egui::State::default())
        // EVENTS
        .add_event::<skin::AddSkinEvent>()
        .add_event::<transform::UpdateSelectionEvent>()
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(DebugLinesPlugin::default())
        // .add_plugin(LogDiagnosticsPlugin::default())
        // .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // STARTUP SYSTEMS
        .add_startup_system(misc::setup)
        .add_startup_system(skin::add_startup_skins)
        .add_startup_system(cloth::create_cloth)
        // SYSTEMS
        // .add_system(add_vertex)
        .add_system(misc::get_mouse_position.label("input_handling"))
        .add_system(misc::update_text)
        .add_system_set(egui::system_set().label("ui_action"))
        .add_system_set(skin::system_set().label("skin_systems"))
        .add_system_set(cloth::system_set().label("update_cloth"))
        .add_system_set(skeleton::system_set().after("skin_systems"))
        .add_system_set(bone::system_set().label("bone_systems"))
        .add_system_set(ccd::system_set().label("ccd_systems"))
        .add_system_set(
            transform::system_set()
                .after("ui_action")
                .after("bone_systems")
                .after("ccd_systems"),
        )
        .add_system_set(
            debug::system_set()
                .after("bone_systems")
                .after("update_cloth"),
        )
        .add_system_set(animation::system_set())
        // RUN
        .run();
}
