mod animation;
mod bone;
mod debug;
mod interpolate;
mod mesh;
mod misc;

use bevy::{
    prelude::*,
    render::mesh::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use debug::DebugDrawer;

const COLOR_SELECTED: Color = Color::rgb(1., 1., 1.);
const COLOR_DEFAULT: Color = Color::rgb(1., 0.6, 0.);

// RESOURCES
pub struct CursorPos(Vec2);

// struct Meshes(Vec<Entity>);
struct Vertices(Vec<Vec2>);

#[derive(Default)]
struct MyMesh {
    handle: Option<Mesh2dHandle>,
    vertices: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    indices: Option<Indices>,
}

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
        .insert_resource(bone::State::new())
        .insert_resource(animation::Animations::new())
        .insert_resource(animation::State::new())
        .insert_resource(DebugDrawer::default())
        .insert_resource(Vertices(Vec::new()))
        .insert_resource(mesh::Skins::default())
        .insert_resource(MyMesh::default())
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        // STARTUP SYSTEMS
        .add_startup_system(misc::setup)
        .add_startup_system(mesh::generate_mesh)
        .add_system(create_textured_mesh)
        // SYSTEMS
        // .add_system(add_vertex)
        .add_system(misc::get_mouse_position.label("input_handling"))
        .add_system_set(bone::system_set())
        .add_system_set(animation::system_set())
        .add_system_set(debug::system_set())
        .add_system(draw_mesh)
        // RUN
        .run();
}

fn draw_mesh(skins: Res<mesh::Skins>, mut debug_drawer: ResMut<DebugDrawer>) {
    // for vertex in mesh.vertices.iter() {
    //     debug_drawer.square(Vec2::from_slice(vertex), 8, COLOR_DEFAULT);
    // }
    // let mut i = 0;
    // while i < mesh.indices.len()-2 {
    //     debug_drawer.line(Vec2::from_slice(&mesh.vertices[mesh.indices[i] as usize]), Vec2::from_slice(&mesh.vertices[mesh.indices[i+1] as usize]), COLOR_DEFAULT);
    //     debug_drawer.line(Vec2::from_slice(&mesh.vertices[mesh.indices[i+1] as usize]), Vec2::from_slice(&mesh.vertices[mesh.indices[i+2] as usize]), COLOR_DEFAULT);
    //     debug_drawer.line(Vec2::from_slice(&mesh.vertices[mesh.indices[i+2] as usize]), Vec2::from_slice(&mesh.vertices[mesh.indices[i] as usize]), COLOR_DEFAULT);
    //     i += 3;
    // }
}

// fn add_vertex(
//     mut vertices: ResMut<Vertices>,
//     mut my_mesh: Res<MyMesh>,
//     cursor_pos: Res<CursorPos>,
//     mouse: Res<Input<MouseButton>>,
//     mut debug_drawer: ResMut<DebugDrawer>,
// ) {
//     if mouse.just_pressed(MouseButton::Right) {
//         vertices.0.push(cursor_pos.0);
//     }
//     debug_drawer.square(cursor_pos.0, 8, COLOR_DEFAULT);
// }

fn create_textured_mesh(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut my_mesh: ResMut<MyMesh>,
    skins: Res<mesh::Skins>,
) {
    // for _vertex in mesh.vertices.iter() {
    //     normals.push([0.,0.,1.]);
    //     uvs.push([0.5,0.5,0.]);
    // }

    // if !mouse.just_pressed(MouseButton::Left) {
    //     return;
    // }

    // my_mesh.vertices = vec![
    //     [0., 0., 0.],
    //     [100., 0., 0.],
    //     [100., 100., 0.],
    //     [cursor_pos.0.x, cursor_pos.0.y, 0.],
    // ];
    // my_mesh.normals = vec![[0., 0., 0.], [0., 0., 0.], [0., 0., 0.], [0., 0., 0.]];
    // my_mesh.uvs = vec![[0., 0.], [1., 0.], [1., 1.], [0., 1.]];
    // my_mesh.indices = Some(Indices::U16(vec![3, 2, 1, 1, 0, 3]));

    let skin = &skins.vec[0];
    my_mesh.vertices = vec![];
    my_mesh.normals = vec![];
    my_mesh.uvs = vec![];
    for vertex in skin.vertices.iter() {
        my_mesh.vertices.push(vertex.clone());
        my_mesh.normals.push([0.,0.,1.]);
        // Vec2::from_slice(vertex).clamp(Vec2::new(0.,0.),Vec2::(239.,239.));
        my_mesh.uvs.push([vertex[0] / skin.dimensions[0] as f32, 1. - vertex[1] / skin.dimensions[1] as f32]);
    }
    my_mesh.vertices[12] = [cursor_pos.0.x,cursor_pos.0.y,0.];
    dbg!(&skin.dimensions);
    dbg!(&my_mesh.uvs);
    let mut inds = skin.indices.clone();
    inds.reverse();
    my_mesh.indices = Some(Indices::U16(inds));

    match my_mesh.handle.clone() {
        Some(mesh_handle) => {
            let _mesh = meshes.get_mut(&mesh_handle.0).unwrap();
            _mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_mesh.vertices.clone());
            _mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, my_mesh.normals.clone());
            _mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
            _mesh.set_indices(my_mesh.indices.clone());
        }
        None => {
            let mut textured_mesh = Mesh::new(PrimitiveTopology::TriangleList);
            textured_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_mesh.vertices.clone());
            textured_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, my_mesh.normals.clone());
            textured_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
            textured_mesh.set_indices(my_mesh.indices.clone());

            let handle: Mesh2dHandle = meshes.add(textured_mesh).into();
            my_mesh.handle = Some(handle.clone());

            commands.spawn_bundle(MaterialMesh2dBundle {
                mesh: handle,
                // transform: Transform::default().with_scale(Vec3::splat(128.)),
                material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
                ..default()
            });
        }
    }
}
