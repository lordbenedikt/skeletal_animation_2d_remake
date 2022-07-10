use crate::*;
use image::GenericImageView;
use lyon_tessellation::*;
use lyon_tessellation::geometry_builder::simple_builder;
use lyon_tessellation::math::{point, Point};
use lyon_tessellation::path::Path;
use std::cmp::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct Polygon {
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl Polygon {
    fn simplify(&mut self) {
        let keep_one_out_of = 30;
        let mut keep_vertices = vec![];
        for i in 0..(self.edges.len() / keep_one_out_of) {
            keep_vertices.push(self.vertices[self.edges[i * keep_one_out_of][0]]);
        }
        self.edges.clear();
        for i in 0..keep_vertices.len() {
            self.edges.push([i, (i + 1) % keep_vertices.len()]);
        }
        self.vertices = keep_vertices;
    }
}

pub struct MultiPolygon {
    pub polygons: Vec<Polygon>,
}
impl MultiPolygon {
    fn from_mesh(mesh: &Mesh) -> MultiPolygon {
        dbg!(mesh.vertices.len());
        let mut polygons: Vec<Polygon> = vec![];
        let mut first_indices_of_polygons: Vec<usize> = vec![0];
        let mut edges: Vec<[usize; 2]> = mesh.edges.clone();
        let mut index = 0;
        let mut iteration = 0;
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
        // split up into individual polygons
        let mut polygon = Polygon::default();
        for i in 0..first_indices_of_polygons.len() {
            let start = first_indices_of_polygons[i];
            let end = if i + 1 == first_indices_of_polygons.len() {
                mesh.vertices.len()
            } else {
                first_indices_of_polygons[i + 1]
            };
            // add edges
            for j in start..end {
                polygon.vertices.push(mesh.vertices[edges[j][0]].clone());
                if polygon.vertices.len() >= 2 {
                    polygon
                        .edges
                        .push([polygon.vertices.len() - 2, polygon.vertices.len() - 1]);
                }
            }
            // add edge between last and first vertex
            dbg!(i);
            polygon.edges.push([polygon.vertices.len() - 1, 0]);
            

            polygons.push(polygon);
            polygon = Polygon::default();
        }
        MultiPolygon { polygons }
    }
    fn simplify(&mut self) {
        for polygon in self.polygons.iter_mut() {
            polygon.simplify();
        }
    }
}

#[derive(Default)]
pub struct Mesh {
    pub filename: String,
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl Mesh {
    fn order_edges(&mut self) {
        let mut index = 0;
        let mut iteration = 0;
        loop {
            let mut found = false;
            for i in iteration..self.edges.len() {
                let mut edge = self.edges[i];
                if edge[0] == index || edge[1] == index {
                    if edge[1] == index {
                        let other = edge[0];
                        edge[0] = edge[1];
                        edge[1] = other;
                    }
                    self.edges.swap(iteration, i);
                    index = edge[1];
                    iteration += 1;
                    found = true;
                    break;
                }
            }
            if iteration == self.edges.len() {
                break;
            }
            if !found {
                index = self.edges[iteration][0];
            }
        }
    }
    fn simplify(&mut self) {
        let keep_one_out_of = 30;
        let mut keep_vertices = vec![];
        for i in 0..(self.edges.len() / keep_one_out_of) {
            keep_vertices.push(self.vertices[self.edges[i * keep_one_out_of][0]]);
        }
        self.edges.clear();
        for i in 0..keep_vertices.len() {
            self.edges.push([i, (i + 1) % keep_vertices.len()]);
        }
        self.vertices = keep_vertices;
    }
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

fn is_close_to_visible_pixel(x: i32, y: i32, img: &image::DynamicImage, max_dist: i32) -> bool {
    let x = x - max_dist;
    let y = y - max_dist;
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

fn get_contour_from_img(filename: &str) -> Mesh {
    let img = image::open(format!("assets/{}", filename)).expect("File not found!");
    let (w, h) = img.dimensions();
    let max_distance: i32 = 5;
    let (output_w, output_h) = (w + max_distance as u32 * 2, h + max_distance as u32 * 2);

    let mut out = vec![vec![0; output_h as usize]; output_w as usize];

    // generate threshold distance map
    for x in 0..output_w {
        for y in 0..output_h {
            out[x as usize][output_h as usize - y as usize - 1] =
                if is_close_to_visible_pixel(x as i32, y as i32, &img, max_distance) {
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
    for x in 0..output_w - 2 {
        for y in 0..output_h - 2 {
            let x = x as usize;
            let y = y as usize;
            contour_grid[x][y] =
                (out[x][y + 1] << 3) + (out[x + 1][y + 1] << 2) + (out[x + 1][y] << 1) + out[x][y];
            let case = &MARCHING_SQUARE_LOOKUP_TABLE[contour_grid[x][y] as usize];
            let current_pos = Vec2::new(x as f32, y as f32) + Vec2::new(0.5, 0.5);
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

    let mut mesh = Mesh::default();
    mesh.filename = String::from(filename);

    // store all unique vertices in mesh resource
    let mut i = 0;
    for (_, indices) in merged_vertices.iter() {
        for index in indices.iter() {
            original_to_new_index.insert(*index, i);
        }
        mesh.vertices.push(contour_vertices[indices[0]]);
        i += 1;
    }

    // store edges with new index in mesh resource
    for [i0, i1] in contour_edges {
        mesh.edges.push([
            *original_to_new_index.get(&i0).unwrap(),
            *original_to_new_index.get(&i1).unwrap(),
        ]);
    }

    dbg!(contour_vertices.len());
    dbg!(mesh.vertices.len());
    dbg!(mesh.edges.len());

    mesh
}

pub fn generate_mesh(mut mesh: ResMut<Mesh>) {
    let contour = get_contour_from_img("head.png");
    let mut multi_polygon = MultiPolygon::from_mesh(&contour);
    multi_polygon.simplify();

    dbg!(multi_polygon.polygons[0].vertices.len());

    mesh.vertices
        .append(&mut multi_polygon.polygons[0].vertices);
    mesh.edges.append(&mut multi_polygon.polygons[0].edges);
}
