use bevy::utils::HashMap;

use crate::{skin::Skin, *};

#[derive(Default)]
pub struct FrameMaterialHandles(HashMap<String, Handle<ColorMaterial>>);

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(update_loose_skin)
        .with_system(update_loose_skin)
        // .with_system(exchange_images)
}

pub fn get_vertex(mesh: &Mesh, ind: usize) -> [f32; 3] {
    unsafe {
        let vertex_attribute_values = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let ptr = vertex_attribute_values.get_bytes().as_ptr() as *const f32;
        let slice = std::slice::from_raw_parts(ptr.offset((ind * 3) as isize), 3);
        return [slice[0], slice[1], slice[2]];
    }
}

pub fn set_vertex(mesh: &mut Mesh, ind: usize, v: [f32; 3]) {
    unsafe {
        let vertex_attribute_values = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let ptr = vertex_attribute_values.get_bytes().as_ptr() as *mut f32;
        let slice = std::slice::from_raw_parts_mut(ptr.offset((ind * 3) as isize), 3);
        for i in 0..3 {
            slice[i] = v[i];
        }
    }
}

pub fn set_vertices(mesh: &mut Mesh, vertices: Vec<Vec3>) {
    for i in 0..vertices.len() {
        set_vertex(mesh, i, [vertices[i][0], vertices[i][1], vertices[i][2]]);
    }
}

pub fn get_vertices(mesh: &Mesh) -> Vec<Vec3> {
    let mut vertices = vec![];
    unsafe {
        let vertex_attribute_values = mesh.attribute(Mesh::ATTRIBUTE_POSITION).unwrap();
        let ptr = vertex_attribute_values.get_bytes().as_ptr() as *const f32;
        for i in 0..mesh.count_vertices() {
            let slice = std::slice::from_raw_parts(ptr.offset((i * 3) as isize), 3);
            vertices.push(Vec3::from_slice(slice));
        }
    }
    vertices
}

pub fn update_loose_skin(
    skeleton: Res<skeleton::Skeleton>,
    mut meshes: ResMut<Assets<Mesh>>,
    q: Query<(&GlobalTransform, &skin::Skin, Entity)>,
) {
    for (gl_transform, skin, entity) in q.iter() {
        let mut is_part_of_skeleton = false;
        for mapping in skeleton.skin_mappings.iter() {
            if mapping.skin.is_none() {
                continue;
            }
            if mapping.skin.unwrap() == entity {
                is_part_of_skeleton = true;
                break;
            }
        }
        if is_part_of_skeleton {
            continue;
        }
        let vertices = skin.gl_vertices(gl_transform);
        let opt_mesh = meshes.get_mut(&skin.mesh_handle.clone().unwrap().0);
        if let Some(mesh) = opt_mesh {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        }
    }
}

pub fn exchange_images(
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    skeleton: Res<skeleton::Skeleton>,
    keys: Res<Input<KeyCode>>,
    mut q: Query<&mut Skin>,
    mut q_meshes: Query<(&Mesh2dHandle, &mut Handle<ColorMaterial>)>,
    mut material_handles: ResMut<FrameMaterialHandles>,
) {
    for (mut skin) in q.iter_mut() {
        if let Ok(subimage) = skin
            .path
            .split("_")
            .last()
            .unwrap()
            .split(".png")
            .next()
            .unwrap()
            .parse::<usize>()
        {
            let path_without_number = skin.path.split(&format!("_{}.png", subimage)).next().unwrap(); 
            let alt_path = format!("{}_{}.png", path_without_number, (subimage + 1) % 16);

            for (mesh_handle, mut material_handle) in q_meshes.iter_mut() {
                if skin.mesh_handle.clone().unwrap().0 == mesh_handle.0 {
                    *material_handle = if let Some(mat_handle) = material_handles.0.get(&alt_path) {
                        mat_handle.clone()
                    } else {
                        let new_handle =
                            materials.add(ColorMaterial::from(asset_server.load(&alt_path)));
                        material_handles
                            .0
                            .insert(alt_path.clone(), new_handle.clone());
                        new_handle
                    };
                }
            }

            skin.path = alt_path;
        } else {
            continue;
        }
    }
}
