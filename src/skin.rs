use crate::*;
use bevy::sprite::MaterialMesh2dBundle;
use cloth::Cloth;
use image::GenericImageView;
use lyon::lyon_tessellation::{
    geometry_builder::simple_builder,
    math::{point, Point},
    path::Path,
    FillGeometryBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use skeleton::Skeleton;
use std::collections::HashMap;
use std::{cmp::*, f32::consts::SQRT_2};

const PIXEL_TO_UNIT_RATIO: f32 = 0.005;
pub const START_SCALE: f32 = 3.5;
pub const AVAILABLE_IMAGES: [&str;7] = [
    "pooh.png",
    "honey.png",
    "head.png",
    "torso.png",
    "left_arm.png",
    "right_arm.png",
    "left_leg.png",
];

#[derive(Default)]
pub struct State {
    pub queued_skins: Vec<AddSkinEvent>,
}

#[derive(Clone)]
pub struct AddSkinEvent {
    pub path: String,
    pub cols: u16,
    pub rows: u16,
    pub as_cloth: bool,
}

#[derive(Default)]
pub struct LineStrip {
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl LineStrip {
    fn simplify(&mut self, filename: &str) {
        let img = image::open(format!("assets/img{}", filename)).expect("File not found!");
        let (w, h) = img.dimensions();

        let mut keep_vertices = vec![self.vertices[self.edges[0][0]]];
        let mut start_index = self.edges[0][0];
        let max_merge_count = 20; // maximum number of merged vertices
        let mut count = 0;
        let mut index = 1;
        while index <= self.edges.len() {
            let _index = index % self.edges.len();
            let current_index = self.edges[_index][0];
            let v0 = self.vertices[start_index];
            let v1 = self.vertices[current_index];
            let magnitude = Vec2::distance(v0, v1) as i32 + 1;
            let normalized = (v1 - v0).normalize();

            let mut is_collision = false;

            // bug, collision is detected to soon
            for i in 1..=magnitude {
                let is_visible = || -> bool {
                    for j in 0..2 {
                        for k in 0..2 {
                            let current_pos = v0 + normalized * i as f32;
                            let x = current_pos.x + j as f32;
                            let y = current_pos.y + k as f32;
                            if x < 0. || x as u32 >= w || y < 0. || y as u32 >= h {
                                continue;
                            }
                            if img.get_pixel(x as u32, h - 1 - y as u32).0[3] > 10 {
                                return true;
                            }
                        }
                    }
                    false
                };
                if is_visible() {
                    is_collision = true;
                    break;
                }
            }
            // let is_collision = index % 40 == 0;

            if is_collision || count >= max_merge_count {
                keep_vertices.push(self.vertices[self.edges[index - 1][0]]);
                start_index = self.edges[index - 1][0];
                count = 0;
            }
            index += 1;
            count += 1;
        }
        self.edges.clear();
        for i in 0..keep_vertices.len() {
            self.edges.push([i, (i + 1) % keep_vertices.len()]);
        }
        self.vertices = keep_vertices;
    }
}

#[derive(Default)]
struct Contour {
    pub filename: String,
    pub dimensions: [u32; 2],
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl Contour {
    fn from_image(
        filename: &str,
        asset_server: &AssetServer,
        image_assets: &Assets<Image>,
        offset: u32,
    ) -> Self {
        let img_handle = asset_server.load(filename);
        let img = image_assets.get(&img_handle).unwrap();
        let size = img.size();
        let (w, h) = (size.x as u32, size.y as u32);

        let max_distance = offset - 1;
        let (output_w, output_h) = (w + offset as u32 * 2, h + offset as u32 * 2);

        let mut out = vec![vec![0; output_h as usize]; output_w as usize];

        // generate threshold distance map
        for x in 0..output_w {
            for y in 0..output_h {
                out[x as usize][output_h as usize - y as usize - 1] =
                    if is_close_to_visible_pixel(x, y, img, offset, max_distance as f32) {
                        1
                    } else {
                        0
                    };
            }
        }

        // generate contour using marching square algorithm
        let mut contour_vertices: Vec<Vec2> = vec![];
        let mut contour_edges: Vec<[usize; 2]> = vec![];
        let mut contour_grid = vec![vec![0_u8; output_h as usize - 1]; output_w as usize - 1];
        for x in 0..output_w - 1 {
            for y in 0..output_h - 1 {
                let x = x as usize;
                let y = y as usize;
                contour_grid[x][y] = (out[x][y + 1] << 3)
                    + (out[x + 1][y + 1] << 2)
                    + (out[x + 1][y] << 1)
                    + out[x][y];
                let case = &MARCHING_SQUARE_LOOKUP_TABLE[contour_grid[x][y] as usize];
                let offset_vector = Vec2::new(offset as f32, offset as f32);
                let current_pos =
                    Vec2::new(x as f32, y as f32) + Vec2::new(0.5, 0.5) - offset_vector;
                // store first edge and vertices
                if case.count > 0 {
                    contour_vertices.push(Vec2::from_slice(&case.vertices[0]) + current_pos);
                    contour_vertices.push(Vec2::from_slice(&case.vertices[1]) + current_pos);
                    let vertex_count: usize = contour_vertices.len();
                    contour_edges.push([vertex_count - 2, vertex_count - 1]);
                }
                // store second edge and vertices
                if case.count > 1 {
                    contour_vertices.push(Vec2::from_slice(&case.vertices[2]) + current_pos);
                    contour_vertices.push(Vec2::from_slice(&case.vertices[3]) + current_pos);
                    let vertex_count: usize = contour_vertices.len();
                    contour_edges.push([vertex_count - 2, vertex_count - 1]);
                }
            }
        }

        let mut merged_vertices: HashMap<u64, Vec<usize>> = HashMap::new();
        let mut original_to_new_index: HashMap<usize, usize> = HashMap::new();

        // insert all unique vertices to hashmap
        for i in 0..contour_vertices.len() {
            let hash = contour_vertices.get(i).unwrap().hash();
            if !merged_vertices.contains_key(&hash) {
                merged_vertices.insert(hash, vec![i]);
            } else {
                merged_vertices.get_mut(&hash).unwrap().push(i);
            }
        }

        let mut graph = Contour::default();
        graph.filename = String::from(filename);
        graph.dimensions = [w, h];

        // store all unique vertices in mesh resource
        let mut i = 0;
        for (_, indices) in merged_vertices.iter() {
            for index in indices.iter() {
                original_to_new_index.insert(*index, i);
            }
            graph.vertices.push(contour_vertices[indices[0]]);
            i += 1;
        }

        // store edges with new index in mesh resource
        for [i0, i1] in contour_edges {
            graph.edges.push([
                *original_to_new_index.get(&i0).unwrap(),
                *original_to_new_index.get(&i1).unwrap(),
            ]);
        }

        graph
    }
}

pub struct Polygon {
    pub filename: String,
    pub dimensions: [u32; 2],
    pub line_strips: Vec<LineStrip>,
}
impl Polygon {
    fn from_contour(contour: &Contour) -> Polygon {
        let mut polygons: Vec<LineStrip> = vec![];
        let mut first_indices_of_polygons: Vec<usize> = vec![0];
        let mut edges: Vec<[usize; 2]> = contour.edges.clone();
        let mut index = 0;
        let mut iteration = 0;

        // order edges
        loop {
            let mut found = false;
            for i in iteration..edges.len() {
                let mut edge = edges[i];
                if edge[0] == index || edge[1] == index {
                    if edge[1] == index {
                        let other = edge[0];
                        edge[0] = edge[1];
                        edge[1] = other;
                    }
                    edges.swap(iteration, i);
                    index = edge[1];
                    iteration += 1;
                    found = true;
                    break;
                }
            }
            if iteration == edges.len() {
                break;
            }
            if !found {
                index = edges[iteration][0];
                first_indices_of_polygons.push(iteration);
            }
        }

        // split up into contiuous lines
        let mut line_strip = LineStrip::default();
        for i in 0..first_indices_of_polygons.len() {
            let start = first_indices_of_polygons[i];
            let end = if i + 1 == first_indices_of_polygons.len() {
                contour.vertices.len()
            } else {
                first_indices_of_polygons[i + 1]
            };
            // add edges
            for j in start..end {
                line_strip
                    .vertices
                    .push(contour.vertices[edges[j][0]].clone());
                if line_strip.vertices.len() >= 2 {
                    line_strip
                        .edges
                        .push([line_strip.vertices.len() - 2, line_strip.vertices.len() - 1]);
                }
            }
            // add edge between last and first vertex
            line_strip.edges.push([line_strip.vertices.len() - 1, 0]);

            polygons.push(line_strip);
            line_strip = LineStrip::default();
        }

        Polygon {
            filename: contour.filename.clone(),
            dimensions: contour.dimensions.clone(),
            line_strips: polygons,
        }
    }
    fn simplify(&mut self) {
        for line_strip in self.line_strips.iter_mut() {
            line_strip.simplify(&self.filename);
        }
    }
    fn triangulate(&self) -> Skin {
        // Create a simple path.
        let mut path_builder = Path::builder();
        let mut is_beginning = true;
        let mut first_vertex = Vec2::new(0., 0.);
        for line_strip in self.line_strips.iter() {
            first_vertex = line_strip.vertices[0];
            if is_beginning {
                path_builder.begin(point(first_vertex.x, first_vertex.y));
                is_beginning = false;
            } else {
                path_builder.end(true);
                path_builder.begin(point(first_vertex.x, first_vertex.y));
            }
            for i in 1..line_strip.vertices.len() {
                let vertex = line_strip.vertices[i];
                path_builder.line_to(point(vertex.x, vertex.y));
            }
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

        let img = image::open(format!("assets/{}", self.filename)).expect("File not found!");
        let (w, h) = img.dimensions();
        let mut vertices: Vec<[f32; 3]> = vec![];
        let mut uvs: Vec<[f32; 2]> = vec![];
        let mut indices: Vec<u16> = vec![];
        for vertex in buffers.vertices {
            vertices.push([
                (vertex.x - (w as f32 / 2.)) * PIXEL_TO_UNIT_RATIO,
                (vertex.y - (h as f32 / 2.)) * PIXEL_TO_UNIT_RATIO,
                0.,
            ]);
            uvs.push([vertex.x / w as f32, 1. - vertex.y / h as f32]);
        }
        for index in buffers.indices {
            indices.push(index);
        }

        Skin {
            path: self.filename.clone(),
            dimensions: self.dimensions.clone(),
            vertices,
            uvs,
            indices,
            ..Default::default()
        }
    }
}

#[derive(Default, Component)]
pub struct Skin {
    pub path: String,
    pub dimensions: [u32; 2],
    pub vertices: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
    pub mesh_handle: Option<Mesh2dHandle>,
    pub depth: f32,
}
impl Skin {
    pub fn gl_vertices(&self, gl_transform: &GlobalTransform) -> Vec<[f32; 3]> {
        let (gl_scale, gl_rotation, gl_translation) = gl_transform.to_scale_rotation_translation();
        self.vertices
            .iter()
            .map(|v| {
                let mut res = Vec3::from_slice(v);
                res *= gl_scale;
                res = Quat::mul_vec3(gl_rotation, res);
                res += gl_translation;
                [res.x, res.y, self.depth]
            })
            .collect::<Vec<[f32; 3]>>()
    }
    pub fn grid_mesh(
        path: &str,
        asset_server: &AssetServer,
        image_assets: &Assets<Image>,
        cols: u16,
        rows: u16,
        depth: f32,
        rectangular: bool,
    ) -> Option<Skin> {
        let img_handle = asset_server.load(path);
        let opt_img = image_assets.get(&img_handle);

        if let Some(img) = opt_img {
            let size = img.size();
            let (w, h) = (size.x as u32, size.y as u32);

            let cell_w = w as f32 / cols as f32;
            let cell_h = h as f32 / rows as f32;
            let mut vertices: Vec<[f32; 3]> = vec![];
            let mut uvs: Vec<[f32; 2]> = vec![];
            for j in (0..=rows).rev() {
                for i in 0..=cols {
                    let uv_pixel = [cell_w * i as f32, cell_h * j as f32];
                    vertices.push([
                        (uv_pixel[0] - w as f32 / 2.) * PIXEL_TO_UNIT_RATIO,
                        (uv_pixel[1] - h as f32 / 2.) * PIXEL_TO_UNIT_RATIO,
                        0.,
                    ]);
                    uvs.push([uv_pixel[0] / w as f32, 1. - uv_pixel[1] / h as f32]);
                }
            }
            let mut indices: Vec<u16> = vec![];
            for j in 0..rows {
                for i in 0..cols {
                    let i0 = j * (cols + 1) + i;
                    let i1 = i0 + 1;
                    let i3 = i0 + (cols + 1);
                    let i2 = i3 + 1;

                    // top left triangle
                    indices.push(i3);
                    indices.push(i0);
                    indices.push(i1);
                    //bottom right triangle
                    indices.push(i1);
                    indices.push(i2);
                    indices.push(i3);

                    // also visible from behind
                    // top left triangle
                    indices.push(i1);
                    indices.push(i0);
                    indices.push(i3);
                    //bottom right triangle
                    indices.push(i3);
                    indices.push(i2);
                    indices.push(i1);
                }
            }
            let mut skin = Skin {
                path: String::from(path),
                dimensions: [w, h],
                vertices,
                uvs,
                indices,
                mesh_handle: None,
                depth,
            };
            // // Remove reduntant vertices and corresponding uvs and indices
            if !rectangular {
                for i in (0..skin.uvs.len()).rev() {
                    let v = skin.uvs[i];
                    let coords = [
                        min((v[0] * w as f32) as u32, w - 1),
                        min((v[1] * h as f32) as u32, h - 1),
                    ];
                    // if uv is out of image or pixel at uv is transparent remove
                    if !is_close_to_visible_pixel(
                        coords[0],
                        coords[1],
                        img,
                        0u32,
                        f32::max(
                            w as f32 / cols as f32 * SQRT_2,
                            h as f32 / rows as f32 * SQRT_2,
                        ),
                    ) {
                        skin.remove_vertex(i as u16);
                    }
                }
            }
            Some(skin)
        } else {
            None
        }
    }
    pub fn remove_vertex(&mut self, index: u16) {
        self.vertices.swap_remove(index as usize);
        self.uvs.swap_remove(index as usize);
        for i in (0..self.indices.len()).step_by(3).rev() {
            if self.indices[i] == index
                || self.indices[i + 1] == index
                || self.indices[i + 2] == index
            {
                for j in (0..3).rev() {
                    self.indices.swap_remove(i + j);
                }
            }
        }
        for i in 0..self.indices.len() {
            if self.indices[i] == self.vertices.len() as u16 {
                self.indices[i] = index;
            }
        }
    }
}

#[derive(Default)]
pub struct Skins {
    pub vec: Vec<Skin>,
}

struct Edges {
    count: u8,
    vertices: [[f32; 2]; 4],
}
impl Edges {
    const fn zero() -> Edges {
        Edges {
            count: 0,
            vertices: [[0., 0.], [0., 0.], [0., 0.], [0., 0.]],
        }
    }
    const fn one(v0: [f32; 2], v1: [f32; 2]) -> Edges {
        Edges {
            count: 1,
            vertices: [v0, v1, [0., 0.], [0., 0.]],
        }
    }
    const fn two(v0: [f32; 2], v1: [f32; 2], v2: [f32; 2], v3: [f32; 2]) -> Edges {
        Edges {
            count: 2,
            vertices: [v0, v1, v2, v3],
        }
    }
}

trait Hash {
    fn hash(&self) -> u64;
}
impl Hash for Vec2 {
    fn hash(&self) -> u64 {
        ((self.x.to_bits() as u64) << 32) + (self.y.to_bits() as u64)
    }
}

const MARCHING_SQUARE_LOOKUP_TABLE: [Edges; 16] = [
    Edges::zero(),
    Edges::one([0., 0.5], [0.5, 0.]),
    Edges::one([0.5, 0.], [1., 0.5]),
    Edges::one([0., 0.5], [1., 0.5]),
    Edges::one([0.5, 1.], [1., 0.5]),
    Edges::two([0., 0.5], [0.5, 1.], [0.5, 0.], [1., 0.5]),
    Edges::one([0.5, 0.], [0.5, 1.]),
    Edges::one([0., 0.5], [0.5, 1.]),
    Edges::one([0., 0.5], [0.5, 1.]),
    Edges::one([0.5, 0.], [0.5, 1.]),
    Edges::two([0., 0.5], [0.5, 0.], [0.5, 1.], [1., 0.5]),
    Edges::one([0.5, 1.], [1., 0.5]),
    Edges::one([0., 0.5], [1., 0.5]),
    Edges::one([0.5, 0.], [1., 0.5]),
    Edges::one([0., 0.5], [0.5, 0.]),
    Edges::zero(),
];

fn is_close_to_visible_pixel(x: u32, y: u32, img: &Image, offset: u32, max_dist: f32) -> bool {
    let max_dist_ceil = f32::ceil(max_dist) as i32;
    let x: i32 = x as i32 - offset as i32;
    let y: i32 = y as i32 - offset as i32;
    let (w, h) = (img.size().x as u32, img.size().y as u32);
    let x_min = max(0, x - max_dist_ceil);
    let x_max = min(w as i32, x + max_dist_ceil);
    for _x in x_min..x_max {
        let y_min = max(0, y - max_dist_ceil);
        let y_max = min(h as i32, y + max_dist_ceil);
        for _y in y_min..y_max {
            let square_distance = (x - _x).pow(2) + (y - _y).pow(2);
            let distance = (square_distance as f32).sqrt();
            // if distance is smaller same max_dist
            if distance <= max_dist {
                // if pixel is not transparent
                if img.get_pixel(_x as u32, _y as u32)[3] > 10 {
                    return true;
                }
            }
        }
    }
    false
}

pub fn generate_mesh(
    path: &str,
    asset_server: &AssetServer,
    image_assets: &Assets<Image>,
) -> Option<Skin> {
    let contour = Contour::from_image(path, asset_server, image_assets, 5);
    let mut polygon = Polygon::from_contour(&contour);
    polygon.simplify();

    let vertices = polygon.line_strips[0]
        .edges
        .iter()
        .map(|edge| polygon.line_strips[0].vertices[edge[0] as usize].clone())
        .collect::<Vec<Vec2>>();
    let mut vertices_split = vec![];
    vertices.iter().for_each(|vertex| {
        vertices_split.push(vertex.x as f64);
        vertices_split.push(vertex.y as f64)
    });

    let mut skin = polygon.triangulate();

    // skin
    Skin::grid_mesh(path, asset_server, image_assets, 40, 40, 0., false)
}

pub fn update_mesh(
    skeleton: Res<Skeleton>,
    mut meshes: ResMut<Assets<Mesh>>,
    q: Query<(&GlobalTransform, &Skin, Entity)>,
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

pub fn create_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut skeleton: ResMut<Skeleton>,
    asset_server: Res<AssetServer>,
    image_assets: &Assets<Image>,
) {
    let opt_skin = skin::generate_mesh("person.png", &asset_server, image_assets);
    if opt_skin.is_none() {
        return;
    }
    let mut skin = opt_skin.unwrap();

    let vertices = skin.vertices.clone();
    let mut normals = vec![];
    let uvs = skin.uvs.clone();
    for _ in skin.vertices.iter() {
        normals.push([0., 1., 1.]);
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
        material: materials.add(ColorMaterial::from(asset_server.load(&skin.path))),
        ..default()
    });
    let skin_id = commands
        .spawn_bundle(TransformBundle::from_transform(Transform {
            scale: Vec3::new(3.5, 3.5, 1.),
            ..Default::default()
        }))
        .insert(Transformable {
            is_selected: false,
            ..default()
        })
        .insert(skin)
        .id();
    skeleton.skin_mappings.push(skeleton::SkinMapping {
        skin: Some(skin_id),
        vertex_mappings: vec![],
    });
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(add_skins)
        .with_system(update_mesh)
}

pub fn add_startup_skins(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<State>,
    asset_server: Res<AssetServer>,
    image_assets: Res<Assets<Image>>,
) {
    state.queued_skins.push(AddSkinEvent { path: String::from("img/honey.png"), cols: 6, rows: 10, as_cloth: true });
    state.queued_skins.push(AddSkinEvent { path: String::from("img/pooh.png"), cols: 30, rows: 30, as_cloth: false });
}

fn add_skin(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
    filename: &str,
    cols: u16,
    rows: u16,
    depth: f32,
    rectangular: bool,
    image_assets: &Assets<Image>,
) -> Option<(Entity, Mesh2dHandle)> {
    let opt_skin = Skin::grid_mesh(
        filename,
        asset_server,
        image_assets,
        cols,
        rows,
        depth,
        rectangular,
    );
    if opt_skin.is_none() {
        return None;
    }
    let mut skin = opt_skin.unwrap();

    let vertices = skin
        .vertices
        .clone()
        .iter()
        .map(|v| [v[0], v[1], depth])
        .collect::<Vec<[f32; 3]>>();
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
        mesh: handle.clone(),
        material: materials.add(ColorMaterial::from(asset_server.load(&skin.path))),
        ..default()
    });
    let skin_id = commands
        .spawn_bundle(TransformBundle::from_transform(Transform {
            scale: Vec3::new(3.5, 3.5, 1.),
            ..default()
        }))
        .insert(Transformable {
            is_selected: false,
            ..default()
        })
        .insert(skin)
        .id();
    Some((skin_id, handle.clone()))
}

pub fn add_skins(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<State>,
    asset_server: Res<AssetServer>,
    image_assets: Res<Assets<Image>>,
) {
    for i in (0..state.queued_skins.len()).rev() {
        let event = &state.queued_skins[i];
        let opt_entity_mesh = add_skin(
            &mut commands,
            &mut meshes,
            &mut materials,
            &asset_server,
            &event.path,
            event.cols,
            event.rows,
            90.,
            event.as_cloth,
            &image_assets,
        );
        if opt_entity_mesh.is_none() {
            break;
        }
        let (entity, mesh_handle) = opt_entity_mesh.unwrap();

        if event.as_cloth {
            let bounding_box = meshes.get(&mesh_handle.0).unwrap().compute_aabb().unwrap();
            let diagonal = (bounding_box.max() - bounding_box.min()) * skin::START_SCALE;
            let cloth = Cloth::new(
                Vec3::new(0., 0., 0.),
                diagonal.x,
                diagonal.y,
                event.cols as usize,
                event.rows as usize,
            )
            .with_stiffness(10);
            commands.entity(entity).insert(cloth);
        }
        state.queued_skins.swap_remove(i);
    }
}

trait Pixels {
    fn get_pixel(&self, x: u32, y: u32) -> &[u8];
    fn get_pixel_alpha(&self, x: u32, y: u32) -> u8;
}
impl Pixels for Image {
    fn get_pixel(&self, x: u32, y: u32) -> &[u8] {
        let (w, _) = (self.size().x as u32, self.size().y as u32);
        let from = (x + y * w) as usize * 4;
        let to = (from + 4) as usize;
        &self.data[from..to]
    }
    fn get_pixel_alpha(&self, x: u32, y: u32) -> u8 {
        let (w, _) = (self.size().x as u32, self.size().y as u32);
        let from = (x + y * w) as usize * 4;
        let alpha = (from + 3) as usize;
        self.data[alpha]
    }
}
