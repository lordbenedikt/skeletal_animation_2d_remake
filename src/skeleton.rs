use bevy::sprite::MaterialMesh2dBundle;

use crate::*;

#[derive(Default)]
pub struct Skeleton {
    pub bones: Vec<Entity>,
    pub skins: Vec<skin::Skin>,
    pub slots: Vec<[usize;2]>,
}

pub fn apply_skin_to_skeleton(
    keys: Res<Input<KeyCode>>,
    skeleton: Res<Skeleton>,
    
) {

}

pub fn create_textured_mesh(
    mut commands: Commands,
    cursor_pos: Res<CursorPos>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut skins: ResMut<skin::Skins>,
) {
    let mut skin = &mut skins.vec[0];
    let mut vertices = vec![];
    let mut normals = vec![];
    let mut uvs = vec![];
    for vertex in skin.vertices.iter() {
        let v_transformed = Quat::mul_vec3(
            Quat::from_rotation_z(skin.rotation),
            Vec3::from_slice(vertex) + skin.offset.extend(0.)
        ) * skin.scale.extend(1.);
        vertices.push([v_transformed.x, v_transformed.y, 0.]);
        normals.push([0.,0.,1.]);
        uvs.push([vertex[0] / skin.dimensions[0] as f32, 1. - vertex[1] / skin.dimensions[1] as f32]);
    }
    let mut inds = skin.indices.clone();
    inds.reverse();
    let indices = Some(Indices::U16(inds));

    match skin.mesh_handle.clone() {
        Some(mesh_handle) => {
            let _mesh = meshes.get_mut(&mesh_handle.0).unwrap();
            _mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
            _mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
            // _mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, my_mesh.uvs.clone());
            _mesh.set_indices(indices.clone());
        }
        None => {
            let mut textured_mesh = Mesh::new(PrimitiveTopology::TriangleList);
            textured_mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices.clone());
            textured_mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals.clone());
            textured_mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs.clone());
            textured_mesh.set_indices(indices.clone());

            let handle: Mesh2dHandle = meshes.add(textured_mesh).into();
            skin.mesh_handle = Some(handle.clone());

            commands.spawn_bundle(MaterialMesh2dBundle {
                mesh: handle,
                // transform: Transform::default().with_scale(Vec3::splat(0.005)),
                material: materials.add(ColorMaterial::from(asset_server.load(&skin.filename))),
                ..default()
            });
        }
    }
}
