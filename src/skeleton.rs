use bevy::sprite::MaterialMesh2dBundle;

use crate::*;
use bone::Bone;
use skin::Skin;

#[derive(Default)]
pub struct Skeleton {
    pub bones: Vec<Entity>,
    pub skins: Vec<Entity>,
    pub vertex_weights: Vec<Vec<Weighting>>,
    pub slots: Vec<[usize; 2]>,
}

pub struct Weighting {
    weights: Vec<f32>,
    entities: Vec<Entity>,
}
impl Weighting {
    fn normalize(&mut self) {
        let mut total = 0.;
        for weight in self.weights.iter() {
            total += *weight;
        }
        for mut weight in self.weights.iter_mut() {
            *weight /= total;
        }
    }
    fn clear(&mut self) {
        self.weights.clear();
        self.entities.clear();
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
    //     .with_system(update_mesh)
    //     .with_system(create_mesh)
}

pub fn assign_skins_to_bones(
    // keys: Res<Input<KeyCode>>,
    mut skeleton: ResMut<Skeleton>,
    q0: Query<(&Transform, &Skin)>,
    q1: Query<(&GlobalTransform, &Bone)>,
) {
    for skin_index in 0..skeleton.skins.len() {
        match q0.get(skeleton.skins[skin_index]) {
            Ok(result) => {
                let skin = result.1;
                for i in 0..skin.vertices.len() {
                    skeleton.vertex_weights[skin_index][i].clear();
                    for bone_index in 0..skeleton.bones.len() {
                        match q1.get(skeleton.bones[bone_index].clone()) {
                            Ok(result) => {
                                let bone_index = skeleton.bones[bone_index];
                                let weighting = &mut skeleton.vertex_weights[skin_index][i];
                                weighting.entities.push(bone_index);
                                weighting.weights.push(10.);
                            }
                            Err(_) => continue,
                        };
                    }
                    skeleton.vertex_weights[skin_index][i].normalize();
                }
            }
            Err(_) => continue,
        };
    }
}

pub fn apply_skin_to_skeleton(
    keys: Res<Input<KeyCode>>,
    skeleton: Res<Skeleton>,
    q: Query<(&GlobalTransform, &Skin)>,
) {
}

pub fn update_mesh(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    q: Query<(&GlobalTransform, &Skin)>,
) {
    for (gl_transform, skin) in q.iter() {
        let vertices = skin.gl_vertices(gl_transform);

        let mesh = meshes.get_mut(skin.mesh_handle.clone().unwrap().0).unwrap();
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
    }
}

pub fn create_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let mut skin = skin::generate_mesh("left_leg.png");

    let vertices = skin.vertices.clone();
    let mut normals = vec![];
    let uvs = skin.uvs.clone();
    for _ in skin.vertices.iter() {
        normals.push([0., 0., 1.]);
    }
    let mut inds = skin.indices.clone();
    inds.reverse();
    let indices = Some(Indices::U16(inds));

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
    mesh.set_indices(indices.clone());

    let handle: Mesh2dHandle = meshes.add(mesh).into();
    skin.mesh_handle = Some(handle.clone());

    commands.spawn_bundle(MaterialMesh2dBundle {
        mesh: handle,
        // transform: Transform::default().with_scale(Vec3::splat(0.005)),
        material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
        ..default()
    });
    commands
    .spawn_bundle(TransformBundle::default())
    .insert(Transformable::default())
    .insert(skin);
}

// pub fn create_textured_mesh(
//     mut commands: Commands,
//     cursor_pos: Res<CursorPos>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<ColorMaterial>>,
//     asset_server: Res<AssetServer>,
// ) {
//     let mut skin = skin::generate_mesh("left_leg.png");
//     let mut normals = vec![];
//     let mut uvs = vec![];
//     for vertex in skin.vertices.iter() {
//         normals.push([0.,0.,1.]);
//         uvs.push([vertex[0] / skin.dimensions[0] as f32, 1. - vertex[1] / skin.dimensions[1] as f32]);
//     }
//     let mut inds = skin.indices.clone();
//     inds.reverse();
//     let indices = Some(Indices::U16(inds));

//     match skin.mesh_handle.clone() {
//         Some(mesh_handle) => {
//             let _mesh = meshes.get_mut(&mesh_handle.0).unwrap();
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skin.vertices.clone());
//             _mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
//             // _mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
//             _mesh.set_indices(indices.clone());
//         }
//         None => {
//             let mut textured_mesh = Mesh::new(PrimitiveTopology::TriangleList);
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, skin.vertices.clone());
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
//             textured_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
//             textured_mesh.set_indices(indices.clone());

//             let handle: Mesh2dHandle = meshes.add(textured_mesh).into();
//             skin.mesh_handle = Some(handle.clone());

//             commands.spawn_bundle(MaterialMesh2dBundle {
//                 mesh: handle,
//                 // transform: Transform::default().with_scale(Vec3::splat(0.005)),
//                 material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
//                 ..default()
//             });
//         }
//     }
// }
