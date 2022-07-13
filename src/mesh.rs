use crate::*;
use image::GenericImageView;
use lyon::lyon_tessellation::{
    geometry_builder::simple_builder,
    math::{point, Point},
    path::Path,
    FillOptions, FillTessellator, VertexBuffers,
};
use std::cmp::*;
use std::collections::HashMap;
use triangle_rs::*;

#[derive(Default)]
pub struct LineStrip {
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl LineStrip {
    fn simplify(&mut self, filename: &str) {
        // dbg!(filename);
        let img = image::open(format!("assets/{}", filename)).expect("File not found!");
        let (w, h) = img.dimensions();

        // let max_skipped_vertices = 100;
        let mut keep_vertices = vec![self.vertices[self.edges[0][0]]];
        let mut start_index = self.edges[0][0];
        let mut index = 1;
        while index <= self.edges.len() {
            let _index = index % self.edges.len();
            let current_index = self.edges[_index][0];
            let v0 = self.vertices[start_index];
            let v1 = self.vertices[current_index];
            let magnitude = Vec2::distance(v0, v1) as i32 + 1;
            let normalized = (v1 - v0).normalize();

            let mut is_collision = false;

            // // bug, collision is detected to soon
            // for i in 1..=magnitude {
            //     let is_visible = || -> bool {
            //         for j in 0..2 {
            //             for k in 0..2 {
            //                 let current_pos = v0 + normalized * i as f32;
            //                 let x = current_pos.x + j as f32;
            //                 let y = current_pos.y + k as f32;
            //                 if x < 0. || x as u32 >= w || y < 0. || y as u32 >= h {
            //                     continue;
            //                 }
            //                 if img.get_pixel(x as u32, y as u32).0[3] > 10 {
            //                     return true;
            //                 }
            //             }
            //         }
            //         false
            //     };
            //     if is_visible() {
            //         is_collision = true;
            //         break;
            //     }
            // }
            let is_collision = index % 40 == 0;

            if is_collision {
                keep_vertices.push(self.vertices[self.edges[index - 1][0]]);
                start_index = self.edges[index - 1][0];
            }
            index += 1;
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
    pub dimensions: [u32;2],
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl Contour {
    fn from_image(filename: &str) -> Self {
        let img = image::open(format!("assets/{}", filename)).expect("File not found!");
        let (w, h) = img.dimensions();
        let offset: i32 = 10;
        let max_distance: i32 = offset - 1;
        let (output_w, output_h) = (w + offset as u32 * 2, h + offset as u32 * 2);

        let mut out = vec![vec![0; output_h as usize]; output_w as usize];

        // generate threshold distance map
        for x in 0..output_w {
            for y in 0..output_h {
                out[x as usize][output_h as usize - y as usize - 1] =
                    if is_close_to_visible_pixel(x as i32, y as i32, &img, offset, max_distance) {
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
        graph.dimensions = [w,h];

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
    pub dimensions: [u32;2],
    pub line_strips: Vec<LineStrip>,
}
impl Polygon {
    fn from_contour(contour: &Contour) -> Polygon {
        // dbg!(contour.vertices.len());
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

        // dbg!(buffers.vertices.len());
        // dbg!(buffers.indices.len());
        // println!("The generated vertices are: {:?}.", &buffers.vertices[..]);
        // println!("The generated indices are: {:?}.", &buffers.indices[..]);

        let mut vertices: Vec<[f32; 3]> = vec![];
        let mut indices: Vec<u16> = vec![];
        for vertex in buffers.vertices {
            vertices.push([vertex.x, vertex.y, 0.]);
        }
        for index in buffers.indices {
            indices.push(index);
        }

        dbg!(&vertices);
        dbg!(&indices);

        Skin {
            filename: self.filename.clone(),
            dimensions: self.dimensions.clone(),
            vertices,
            indices,
        }
    }
}

#[derive(Default)]
pub struct Skin {
    pub filename: String,
    pub dimensions: [u32;2],
    pub vertices: Vec<[f32; 3]>,
    pub indices: Vec<u16>,
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

fn is_close_to_visible_pixel(
    x: i32,
    y: i32,
    img: &image::DynamicImage,
    offset: i32,
    max_dist: i32,
) -> bool {
    let x = x - offset;
    let y = y - offset;
    let (w, h) = img.dimensions();
    let (w, h) = (w as i32, h as i32);
    let x_min = max(0, x as i32 - max_dist);
    let x_max = min(w as i32, x as i32 + max_dist);
    for _x in x_min..x_max {
        let y_min = max(0, y as i32 - max_dist);
        let y_max = min(h as i32, y as i32 + max_dist);
        for _y in y_min..y_max {
            let square_distance = (x as i32 - _x).pow(2) + (y as i32 - _y).pow(2);
            let distance = (square_distance as f32).sqrt();
            // if distance is smaller same max_dist
            if distance <= max_dist as f32 {
                // if pixel is not transparent
                if img.get_pixel(_x as u32, _y as u32).0[3] > 10 {
                    return true;
                }
            }
        }
    }
    false
}

pub fn generate_mesh(mut skins: ResMut<Skins>, mut debug_drawer: ResMut<DebugDrawer>) {
    let contour = Contour::from_image("left_leg.png");
    let mut polygon = Polygon::from_contour(&contour);
    polygon.simplify();
    
    let vertices = polygon.line_strips[0].edges.iter().map(|edge| polygon.line_strips[0].vertices[edge[0] as usize].clone()).collect::<Vec<Vec2>>();
    let mut vertices_split = vec![];
    vertices.iter().for_each(|vertex| {vertices_split.push(vertex.x as f64); vertices_split.push(vertex.y as f64)});

    let mut _mesh = polygon.triangulate();

    skins.vec.push(_mesh);
}
