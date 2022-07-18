use crate::*;

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
        set_vertex(mesh, i, [vertices[i][0],vertices[i][1],vertices[i][2]]);
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