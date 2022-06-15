mod cursor;
mod setup;
mod bones;

use bevy::prelude::*;
use bevy_prototype_debug_lines::DebugLinesPlugin;

// RESOURCES
pub struct Bones(Vec<Entity>);
pub struct CursorPos(Vec2);

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
        .insert_resource(Bones(Vec::new()))
        .insert_resource(CursorPos(Vec2::new(0., 0.)))
        .insert_resource(bones::Editor::new())
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::with_depth_test(true))
        // STARTUP SYSTEMS
        .add_startup_system(setup::setup)
        // SYSTEMS
        .add_system(cursor::get_position.label("input_handling"))
        .add_system_set(bones::system_set())
        // RUN
        .run();
}
