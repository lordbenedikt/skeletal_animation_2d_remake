mod animation;
mod bone;
mod debug;
mod interpolate;
mod mesh;
mod misc;

use bevy::{prelude::*, render::mesh::*, sprite::MaterialMesh2dBundle};
use bevy_prototype_debug_lines::{DebugLines, DebugLinesPlugin};
use debug::DebugDrawer;
use lyon_tessellation::{
    geometry_builder::simple_builder,
    math::{point, Point},
    path::Path,
    FillOptions, FillTessellator, VertexBuffers,
};

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
        .insert_resource(bone::State::new())
        .insert_resource(animation::Animations::new())
        .insert_resource(animation::State::new())
        .insert_resource(DebugDrawer::default())
        .insert_resource(Vertices(Vec::new()))
        .insert_resource(mesh::Mesh::default())
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        // STARTUP SYSTEMS
        .add_startup_system(misc::setup)
        .add_startup_system(mesh::generate_mesh.before(create_textured_mesh))
        // .add_startup_system(create_textured_mesh)
        // SYSTEMS
        .add_system(add_vertex)
        .add_system(misc::get_mouse_position.label("input_handling"))
        .add_system_set(bone::system_set())
        .add_system_set(animation::system_set())
        .add_system_set(debug::system_set())
        .add_system(draw_mesh)
        // RUN
        .run();
}

fn draw_mesh(mesh: Res<mesh::Mesh>, mut debug_drawer: ResMut<DebugDrawer>) {
    for vertex in mesh.vertices.iter() {
        debug_drawer.square(*vertex, 8, COLOR_DEFAULT);
    }
    for [i0, i1] in mesh.edges.iter() {
        debug_drawer.line(mesh.vertices[*i0], mesh.vertices[*i1], COLOR_DEFAULT);
    }
}

fn add_vertex(
    mut vertices: ResMut<Vertices>,
    cursor_pos: Res<CursorPos>,
    mouse: Res<Input<MouseButton>>,
    mut debug_drawer: ResMut<DebugDrawer>,
) {
    if mouse.just_pressed(MouseButton::Right) {
        vertices.0.push(cursor_pos.0);
    }
    debug_drawer.square(cursor_pos.0, 8, COLOR_DEFAULT);
}

fn create_textured_mesh(
    mut commands: Commands,
    mesh: Res<mesh::Mesh>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut vertices: Vec<[f32; 3]> = vec![];
    let mut normals: Vec<[f32; 3]> = vec![];
    let mut uvs: Vec<[f32; 3]> = vec![];
    let mut indices = Indices::U16(vec![]);

    // Create a simple path.
    let mut path_builder = Path::builder();
    let vertex = mesh.vertices[0];
    path_builder.begin(point(vertex.x, vertex.y));
    for i in 1..mesh.vertices.len() {
        let vertex = mesh.vertices[i];
        path_builder.line_to(point(vertex.x, vertex.y));
    }
    path_builder.end(true);
    let path = path_builder.build();

    // Create the destination vertex and index buffers.
    let mut buffers: VertexBuffers<Point, u16> = VertexBuffers::new();

    {
        let mut vertex_builder = simple_builder(&mut buffers);

        // Create the tessellator.
        let mut tessellator = FillTessellator::new();

        // Compute the tessellation.
        let result =
            tessellator.tessellate_path(&path, &FillOptions::default(), &mut vertex_builder);
        assert!(result.is_ok());
    }

    dbg!(buffers.vertices.len());
    dbg!(buffers.indices.len());
    println!("The generated vertices are: {:?}.", &buffers.vertices[..]);
    println!("The generated indices are: {:?}.", &buffers.indices[..]);

    for vertex in buffers.vertices {
        vertices.push([vertex.x, vertex.y, 0.]);
        normals.push([0., 0., 1.]);
        uvs.push([0.5, 0.5, 0.5]);
    }
    indices = Indices::U16(buffers.indices.clone());

    dbg!(&vertices);
    dbg!(&normals);
    dbg!(&uvs);
    dbg!(&indices);

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.set_indices(Some(indices));

    // // let mut camera = OrthographicCameraBundle::new_2d();
    // // camera.orthographic_projection.scale = 1f32;

    // // commands.spawn_bundle(camera);
    commands.spawn_bundle(MaterialMesh2dBundle {
        
        mesh: meshes.add(mesh).into(),
        transform: Transform::default().with_scale(Vec3::splat(128.)),
        material: materials.add(ColorMaterial::from(asset_server.load("Unbenannt.png"))),
        ..default()
    });
}
