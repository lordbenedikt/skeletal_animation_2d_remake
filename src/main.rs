mod animation;
mod bone;
mod debug;
mod interpolate;
mod mesh;
mod misc;

use bevy::{prelude::*, render::mesh::*, sprite::{MaterialMesh2dBundle, Mesh2dHandle}};
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
    vertices: Vec<[f32;3]>,
    normals: Vec<[f32;3]>,
    uvs: Vec<[f32;2]>,
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
        .insert_resource(mesh::Mesh::default())
        // .insert_resource(MyMesh::default())
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        // STARTUP SYSTEMS
        .add_startup_system(misc::setup)
        .add_startup_system(mesh::generate_mesh)
        // .add_startup_system(create_textured_mesh)
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

fn draw_mesh(mesh: Res<mesh::Mesh>, mut debug_drawer: ResMut<DebugDrawer>) {
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

// fn create_textured_mesh(
//     mut commands: Commands,
//     mesh: Res<mesh::Mesh>,
//     mut my_mesh: ResMut<MyMesh>,
//     mouse: Res<Input<MouseButton>>,
//     cursor_pos: Res<CursorPos>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     // let mut normals: Vec<[f32; 3]> = vec![];
//     // let mut uvs: Vec<[f32; 3]> = vec![];

//     // dbg!(&mesh.vertices);
//     // dbg!(&mesh.indices);

//     // for _vertex in mesh.vertices.iter() {
//     //     normals.push([0.,0.,1.]);
//     //     uvs.push([0.5,0.5,0.]);
//     // }

//     if !mouse.just_pressed(MouseButton::Left) {
//         return;
//     }

//     match my_mesh.handle.clone() {
//         Some(mesh_handle) => {
//             my_mesh.vertices.push([cursor_pos.0.x, cursor_pos.0.y, 0.]);
//             my_mesh.normals.push([0.,0.,1.]);
//             my_mesh.uvs.push([0.5,0.5]);
//             if my_mesh.vertices.len() >=3 {
//                 let mut indices = vec![];
//                 for i in 2..my_mesh.vertices.len() {
//                     indices.push(i-2);
//                     indices.push(i-1);
//                     indices.push(i);
//                 }
//                 my_mesh.indices = Some(Indices::U16(vec![]));
//             }

//             let _mesh = meshes.get_mut(&mesh_handle.0).unwrap();
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_mesh.vertices.clone());
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, my_mesh.normals.clone());
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
//             _mesh.set_indices(my_mesh.indices.clone());
//         },
//         None => {
//             my_mesh.vertices.push([cursor_pos.0.x, cursor_pos.0.y, 0.]);
//             my_mesh.normals.push([0.,0.,1.]);
//             my_mesh.uvs.push([0.5,0.5]);
//             my_mesh.indices = Some(Indices::U16(vec![]));

//             let mut textured_mesh = Mesh::new(PrimitiveTopology::TriangleList);
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, my_mesh.vertices.clone());
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, my_mesh.normals.clone());
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
//             textured_mesh.set_indices(my_mesh.indices.clone());

//             my_mesh.handle = Some(meshes.add(textured_mesh).into()); 
            
//             commands.spawn_bundle(MaterialMesh2dBundle {
//                 mesh: my_mesh.handle,
//                 // transform: Transform::default().with_scale(Vec3::splat(128.)),
//                 material: materials.add(ColorMaterial::from(asset_server.load("left_leg.png"))),
//                 ..default()
//             });
//         },
//     }


    
    // let mut textured_mesh = Mesh::new(PrimitiveTopology::TriangleList);
    // textured_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, mesh.vertices.clone());
    // textured_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    // textured_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    // textured_mesh.set_indices(Some(Indices::U16(mesh.indices.clone())));

    // my_mesh = meshes.add(textured_mesh).into();


// }
