mod animation;
mod bone;
mod ccd;
mod cloth;
mod debug;
mod egui;
mod interpolate;
mod mesh;
mod misc;
mod save_load;
mod skeleton;
mod skin;
mod transform;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{prelude::*, render::mesh::*, sprite::Mesh2dHandle};
use bevy_egui::EguiPlugin;
use bevy_prototype_lyon::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use debug::DebugDrawer;
use transform::*;
use wasm_bindgen::prelude;
use web_sys::*;

const COLOR_WHITE: Color = Color::rgb(1., 1., 1.);
const COLOR_GRAY: Color = Color::rgb(0.3, 0.3, 0.3);
const COLOR_BLACK: Color = Color::rgb(0., 0., 0.);
const COLOR_SELECTED: Color = Color::rgb(1., 0.9, 0.);
const COLOR_DEFAULT: Color = Color::rgb(1., 0.6, 0.);
const COLOR_SELECTED_ACTIVE: Color = Color::rgb(0.7, 0.7, 1.);
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

    // RESOURCES
    // general resources
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
    // state resources
    .insert_resource(animation::State::new())
    .insert_resource(skin::State::default())
    .insert_resource(egui::State::default())
    .insert_resource(cloth::State::default())
    .insert_resource(save_load::State::default())
    // EVENTS
    .add_event::<animation::ShowKeyframeEvent>()
    // PLUGINS
    .add_plugins(DefaultPlugins)
    .add_plugin(ShapePlugin)
    .add_plugin(EguiPlugin)
    .add_plugin(JsonAssetPlugin::<save_load::CompleteJson>::new(&["anim"]))
    // .add_plugin(LogDiagnosticsPlugin::default())
    // .add_plugin(FrameTimeDiagnosticsPlugin::default())
    // STARTUP SYSTEMS
    .add_startup_system(misc::setup)
    // SYSTEMS
    .add_system(misc::get_mouse_position.label("input_handling"))
    .add_system(misc::update_text)
    .add_system_set(egui::system_set().label("ui_action"))
    .add_system_set(skin::system_set().label("skin_systems"))
    .add_system_set(
        cloth::system_set()
            .label("update_cloth")
            .before("skin_systems"),
    )
    .add_system_set(skeleton::system_set().after("skin_systems"))
    .add_system_set(bone::system_set().label("bone_systems").after("ui_action"))
    .add_system_set(ccd::system_set().label("ccd_systems"))
    .add_system_set(
        debug::system_set()
            .after("bone_systems")
            .after("update_cloth")
            .label("debug_systems"),
    )
    .add_system_set(
        transform::system_set()
            .after("ui_action")
            .after("bone_systems")
            .after("ccd_systems")
            .before("debug_systems"),
    )
    .add_system_set(animation::system_set())
    .add_system_set(save_load::system_set());

    // Don't execute on Web
    #[cfg(target_arch = "wasm32")]
    app.add_system(misc::wasm_resize_window);

    // RUN
    app.run();
}