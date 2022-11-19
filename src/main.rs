mod animation;
mod bevy_image;
mod bevy_ui;
mod bone;
mod inverse_kinematics;
mod cloth;
mod debug;
mod egui;
mod interpolate;
mod mesh;
mod mesh_gen;
mod misc;
mod save_load;
mod skeleton;
mod skin;
mod transform;
mod kinematic_chain;

#[cfg(test)]
#[path = "tests/assert.rs"]
mod assert;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::mesh::*, sprite::Mesh2dHandle};
use bevy_common_assets::json::JsonAssetPlugin;
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::*;
use debug::DebugDrawer;
use transform::*;
use wasm_bindgen::prelude;
use web_sys::*;

const COLOR_WHITE: Color = Color::rgb(1., 1., 1.);
const COLOR_GRAY: Color = Color::rgb(0.3, 0.3, 0.3);
const COLOR_LIGHT_GRAY: Color = Color::rgb(0.55, 0.55, 0.55);
const COLOR_LIGHTER_GRAY: Color = Color::rgb(0.7, 0.7, 0.7);
const COLOR_RED: Color = Color::rgb(1.0, 0.0, 0.0);
const COLOR_RED_TRANSPARENT: Color = Color::rgba(1.0, 0.0, 0.0, 0.4);
const COLOR_GREEN: Color = Color::rgb(0.0, 1.0, 0.0);
const COLOR_GREEN_TRANSPARENT: Color = Color::rgba(0.0, 1.0, 0.0, 0.4);
const COLOR_BLACK: Color = Color::rgb(0., 0., 0.);
const COLOR_SELECTED: Color = Color::rgb(1., 0.9, 0.);
const COLOR_DEFAULT: Color = Color::rgb(1., 0.6, 0.);
const COLOR_SELECTED_ACTIVE: Color = Color::rgb(0., 0.9, 1.);
const COLOR_DEFAULT_ACTIVE: Color = Color::rgb(0.2, 0.2, 1.);

const PIXELS_PER_UNIT: u32 = 100;

// RESOURCES
pub struct CursorPos(Vec2);

#[derive(Default)]
pub struct General {
    done: bool,
}

fn main() {
    let mut app = App::new();

    // GENERAL RESOURCES
    app.insert_resource(WindowDescriptor {
        title: "Skeletal Animation".to_string(),
        // width: 800.,
        // height: 600.,
        mode: bevy::window::WindowMode::BorderlessFullscreen,
        ..Default::default()
    })
    .insert_resource(ClearColor(COLOR_GRAY))
    .insert_resource(CursorPos(Vec2::new(0., 0.)))
    .insert_resource(transform::State::new())
    .insert_resource(animation::Animations::new())
    .insert_resource(DebugDrawer::default())
    .insert_resource(skin::Skins::default())
    .insert_resource(skeleton::Skeleton::default())
    .insert_resource(General::default())
    .insert_resource(bevy_ui::UiElements::default())
    .insert_resource(mesh::FrameMaterialHandles::default())
    // STATE RESOURCES
    .insert_resource(animation::State::new())
    .insert_resource(skin::State::default())
    .insert_resource(egui::State::default())
    .insert_resource(cloth::State::default())
    .insert_resource(save_load::State::default())
    // EVENTS
    .add_event::<animation::ShowKeyframeEvent>()
    .add_event::<save_load::SaveEvent>()
    .add_event::<save_load::LoadEvent>()
    // PLUGINS
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    .add_plugin(EguiPlugin)
    .add_plugin(JsonAssetPlugin::<save_load::CompleteJson>::new(&["anim"]))
    // LOG DIAGNOSTICS
    // .add_plugin(LogDiagnosticsPlugin::default())
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    // STARTUP SYSTEMS
    .add_startup_system(misc::setup)
    .add_startup_system(bevy_ui::spawn_ui_elements)
    // SYSTEMS
    .add_system(misc::get_mouse_position.label("input_handling"))
    .add_system_set(bevy_ui::system_set())
    .add_system_set(egui::system_set().label("ui_action"))
    .add_system_set(skin::system_set().label("skin_systems"))
    .add_system_set(mesh::system_set().label("mesh_systems"))
    .add_system_set(bone::system_set().label("bone_systems").after("ui_action"))
    .add_system_set(animation::system_set().label("animation_systems"))
    .add_system_set(
        transform::system_set()
            .label("transform_systems")
            .after("ui_action")
            .after("bone_systems")
            .after("animation_systems"),
    )
    .add_system_set(
        cloth::system_set()
            .label("update_cloth")
            .before("mesh_systems")
            .after("animation_systems"),
    )
    .add_system_set(
        inverse_kinematics::system_set()
            .label("ccd_systems")
            .after("transform_systems")
            .after("animation_systems"),
    )
    .add_system_set(
        skeleton::system_set()
            .after("mesh_systems")
            .after("ccd_systems")
            .after("animation_systems")
            .label("skeleton_systems"),
    )
    .add_system_set(
        debug::system_set()
            .after("bone_systems")
            .after("update_cloth")
            .after("ccd_systems")
            .after("skeleton_systems")
            .label("debug_systems"),
    )
    .add_system_set(save_load::system_set());

    // Don't execute on Web
    #[cfg(target_arch = "wasm32")]
    app.add_system(misc::wasm_resize_window);

    // RUN
    app.run();
}
