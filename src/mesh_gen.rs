use bevy::utils::{HashMap, HashSet};
use geo::*;
use misc::*;
use spade::*;

use crate::{image::Pixels, *};

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

#[derive(Default)]
pub struct Contour {
    pub path: String,
    pub img: Image,
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl Contour {
    pub fn to_mesh(&self, triangle_size: f32) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<u16>) {
        let multipoly = self.to_multipoly(triangle_size as u32);
        let cdt = multipoly
            .triangulate_delaunay(triangle_size)
            .expect("delaunay triangulation error");

        let mut vertices = vec![];
        let mut indices = vec![];
        let mut unique_vertices: HashMap<u64, u16> = HashMap::new();

        // Extract vertices and indices from cdt
        for face in cdt.inner_faces() {
            for v in face.vertices() {
                let key = vec2_to_u64(Vec2::new(v.position().x, v.position().y));
                let index: u16;
                if unique_vertices.contains_key(&key) {
                    index = unique_vertices[&key];
                } else {
                    index = unique_vertices.len() as u16;
                    unique_vertices.insert(key, index);
                    vertices.push([v.position().x, v.position().y, 0.]);
                }
                indices.push(index);
            }
        }

        // Remove triangles that are outside multipolygon
        for i in ((0..indices.len()).step_by(3)).rev() {
            let mut center = Vec2::new(0., 0.);
            for j in 0..3 {
                let v = vertices[indices[i + j] as usize];
                center.x += v[0];
                center.y += v[1];
            }
            center /= 3.;
            if !multipoly.intersects(&Coordinate {
                x: center.x,
                y: center.y,
            }) {
                for j in (0..3).rev() {
                    indices.remove(i + j); // Swap removed could be used, but changes triangle ordering
                }
            }
        }

        let (w, h) = (self.img.size().x, self.img.size().y);
        let mut uvs: Vec<[f32; 2]> = vec![];
        for v in vertices.iter() {
            let x = v[0] / w;
            let y = 1. - v[1] / h;
            uvs.push([x, y]);
        }

        for i in 0..vertices.len() {
            vertices[i][0] -= w / 2.0;
            vertices[i][1] -= h /2.0;
            for j in 0..3 {
                vertices[i][j] *= skin::PIXEL_TO_UNIT_RATIO;
            }
        }

        (vertices, uvs, indices)
    }

    pub fn from_image(
        path: &str,
        asset_server: &AssetServer,
        image_assets: &Assets<Image>,
        borderline_width: i32,
    ) -> Option<Self> {
        let img_handle = asset_server.load(path);
        let opt_img = image_assets.get(&img_handle);
        let img = if opt_img.is_some() {
            opt_img.unwrap()
        } else {
            println!("loading image");
            return None;
        };
        let size = img.size();
        let (w, h) = (size.x as i32, size.y as i32);

        let offset = borderline_width + 1;
        let max_distance = borderline_width;
        let (distance_field_w, distance_field_h) = (w + offset * 2, h + offset * 2);

        let mut out = vec![vec![0; distance_field_h as usize]; distance_field_w as usize];

        // generate threshold distance field
        for output_x in 0..distance_field_w {
            for output_y in 0..distance_field_h {
                out[output_x as usize][distance_field_h as usize - output_y as usize - 1] = if img
                    .is_close_to_visible(output_x - offset, output_y - offset, max_distance as f32)
                {
                    1
                } else {
                    0
                };
            }
        }

        // generate contour using marching square algorithm
        let mut contour_vertices: Vec<Vec2> = vec![];
        let mut contour_edges: Vec<[usize; 2]> = vec![];
        let mut contour_grid =
            vec![vec![0_u8; distance_field_h as usize - 1]; distance_field_w as usize - 1];
        for x in 0..distance_field_w - 1 {
            for y in 0..distance_field_h - 1 {
                let x = x as usize;
                let y = y as usize;
                contour_grid[x][y] = (out[x][y + 1] << 3)
                    + (out[x + 1][y + 1] << 2)
                    + (out[x + 1][y] << 1)
                    + out[x][y];
                let case = &MARCHING_SQUARE_LOOKUP_TABLE[contour_grid[x][y] as usize];
                let offset_vector = Vec2::new(offset as f32, offset as f32);
                let current_pos =
                    Vec2::new(x as f32, y as f32) + Vec2::new(0.0, 0.5) - offset_vector;
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

        let mut contour = Contour::default();
        contour.path = String::from(path);
        contour.img = img.clone();

        // add vertices to contour
        let mut i = 0;
        for (_, indices) in merged_vertices.iter() {
            for index in indices.iter() {
                original_to_new_index.insert(*index, i);
            }
            contour.vertices.push(contour_vertices[indices[0]]);
            i += 1;
        }

        // add edges with changed indices to contour
        for [i0, i1] in contour_edges {
            contour.edges.push([
                *original_to_new_index.get(&i0).unwrap(),
                *original_to_new_index.get(&i1).unwrap(),
            ]);
        }

        Some(contour)
    }

    fn to_simplified_line_strings(&self, max_edge_len: u32) -> Vec<LineString<f32>> {
        let vertices: &Vec<Vec2> = &self.vertices;
        let mut edges: Vec<[usize; 2]> = self.edges.clone();
        if edges.len() == 0 {
            return vec![];
        }

        // Array containing result
        let mut line_strings: Vec<LineString<f32>> = vec![];
        // Array containing part of result
        let mut line_string: LineString<f32> = LineString::new(vec![]);
        // Start with first vertex of first edge
        let mut next_index: usize = self.edges[0][0];

        // Find and store coordinates in correct order
        'outer: loop {
            for i in 0..edges.len() {
                // Check if one of the vertices is the sought vertex
                let vertex_number: usize = if edges[i][0] == next_index {
                    0
                } else if edges[i][1] == next_index {
                    1
                } else {
                    continue;
                };

                // Push next vec2 to line string
                let v = vec2_as_coord(&vertices[edges[i][vertex_number]]);
                line_string.0.push(v);

                // Set next_index to the index of the neighbouring vertex
                next_index = edges[i][if vertex_number == 0 { 1 } else { 0 }];

                // Remove edge, so that it won't be checked again
                edges.swap_remove(i);

                // Exit for loop
                continue 'outer;
            }

            line_strings.push(line_string);
            line_string = LineString::new(vec![]);
            if !edges.is_empty() {
                next_index = edges[0][0];
            } else {
                break 'outer;
            }
        }
        self.simplify_line_strings(line_strings, max_edge_len)
    }

    fn simplify_line_strings(
        &self,
        line_strings: Vec<LineString<f32>>,
        max_edge_len: u32,
    ) -> Vec<LineString<f32>> {
        let mut res = vec![LineString::<f32>::new(vec![]); line_strings.len()];

        // Search one by one for maximum number of edges that can be merged
        for index in 0..line_strings.len() {
            if line_strings[index].0.len() < 3 {
                continue;
            }
            let mut start = 0;
            let mut end = 0;
            for i in (start + 1)..=line_strings[index].0.len() {
                let a_coord = line_strings[index].0[start];
                let b_coord = line_strings[index].0[i % line_strings[index].0.len()];
                let a_vec2 = Vec2::new(a_coord.x, a_coord.y);
                let b_vec2 = Vec2::new(b_coord.x, b_coord.y);
                if self.intersects_visible((a_vec2, b_vec2))
                    || end - start == max_edge_len as usize
                    || i == line_strings[index].0.len()
                {
                    res[index].0.push(line_strings[index].0[start]);
                    start = end;
                }
                end = i;
            }
        }

        for i in (0..res.len()).rev() {
            if res[i].0.len() < 3 {
                res.swap_remove(i);
            }
        }

        // // Binary search for maximum number of edges that can be merged
        // // THIS DOESN'T WORK, as it can lead to circling an area of the image without detecting collision
        // for index in 0..line_strings.len() {
        //     let mut a: usize = 0;
        //     let mut n: usize = min(max_merge_size, line_strings[index].0.len() - 1);
        //     let mut m: usize = n;
        //     while a < line_strings[index].0.len() - 1 {
        //         println!("while...");
        //         let a_coord = line_strings[index].0[a];
        //         let b_coord = line_strings[index].0[n];
        //         let a_vec2 = Vec2::new(a_coord.x, a_coord.y);
        //         let b_vec2 = Vec2::new(b_coord.x, b_coord.y);
        //         if !self.intersects_visible((a_vec2, b_vec2)) || m==a+1 {
        //             if m == n || m == a+1 {
        //                 // Found!!
        //                 res[index].0.push(a_coord);
        //                 a = m;
        //                 n = cmp::min(line_strings[index].0.len() - 1, m + max_merge_size);
        //                 m = n;
        //             } else {
        //                 println!("rasing..");
        //                 m = ((m + n) as f32 / 2.0 + 0.5) as usize;
        //             }
        //         } else {
        //             println!("reducing.. (a={}, m={}, n={})", a,m,n);
        //             n = m;
        //             m = (a + m) / 2;
        //         }
        //     }
        // }

        res
    }

    fn intersects_visible(&self, line: (Vec2, Vec2)) -> bool {
        let (w, h) = (self.img.size().x as u32, self.img.size().y as u32);
        let a = line.0;
        let b = line.1;
        let magnitude = Vec2::distance(a, b) as i32 + 1;
        let normalized = (b - a).normalize();

        let mut is_collision = false;

        // bug, collision is detected to soon
        for i in 1..=magnitude {
            let is_visible = || -> bool {
                for j in 0..2 {
                    for k in 0..2 {
                        let current_pos = a + normalized * i as f32;
                        let x = current_pos.x + j as f32;
                        let y = current_pos.y + k as f32;
                        if x < 0. || x as u32 >= w || y < 0. || y as u32 >= h {
                            continue;
                        }
                        if self.img.get_pixel(x as u32, h - 1 - y as u32)[3] > 10 {
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

        is_collision
    }

    pub fn to_multipoly(&self, max_edge_len: u32) -> MultiPolygon<f32> {
        let line_strings = self.to_simplified_line_strings(max_edge_len);

        // Setup todo to contain all line string's indices
        let mut todo: Vec<usize> = vec![];
        for i in 0..line_strings.len() {
            todo.push(i);
        }

        // Setup array that will contain all indices of line strings that contain the current one
        let mut contained_by: Vec<HashSet<usize>> = vec![HashSet::new(); line_strings.len()];
        for i in 0..line_strings.len() {
            // Check whether polygon defined by line string i contains line string j
            let poly = geo::Polygon::new(line_strings[i].clone(), vec![]);
            for j in 0..line_strings.len() {
                if i == j {
                    continue;
                } else {
                    if poly.contains(&line_strings[j]) {
                        // And store the result in contained_by
                        contained_by[j].insert(i);
                    }
                }
            }
        }

        // Array that will contain all polygons making up the multipolygon
        let mut res: Vec<geo::Polygon<f32>> = vec![];

        // Algorithms main loop
        while !todo.is_empty() {

            // Processed line strings
            let mut done = vec![];

            // for loop will always reassign exterior_index
            let mut exterior_index = 0;
            for i in 0..contained_by.len() {
                exterior_index = todo[i];
                if contained_by[i].len() == 0 {
                    res.push(geo::Polygon::new(line_strings[exterior_index].clone(), vec![]));
                    done.push(exterior_index);
                    break;
                }
            }
            
            // Add interiors to polygons
            for i in 0..line_strings.len() {
                // Skip self check
                if i == exterior_index {
                    continue;
                }
                // Push line string, if interior line_string is enclosed by only the exterior line_string
                let set = &contained_by[i];
                if set.len() == 1 && set.contains(&exterior_index) {
                    res.last_mut().unwrap().interiors_push(line_strings[i].clone());
                    done.push(i);
                }
            }

            for i in (0..todo.len()).rev() {
                if done.contains(&todo[i]) {
                    todo.swap_remove(i);
                }
            }
        }
        println!("polygons: {}",res.len());
        MultiPolygon::new(res)
    }
}

pub trait DelaunayTriangulation {
    // fn triangulate(&self, img_dimensions: Vec2, triangle_size: f32) -> Option<Skin>;
    fn triangulate_delaunay(
        &self,
        triangle_size: f32,
    ) -> Result<ConstrainedDelaunayTriangulation<Point2<f32>>, InsertionError>;
}

impl DelaunayTriangulation for MultiPolygon<f32> {
    fn triangulate_delaunay(
        &self,
        triangle_size: f32,
    ) -> Result<ConstrainedDelaunayTriangulation<Point2<f32>>, InsertionError> {
        let mut cdt = ConstrainedDelaunayTriangulation::<Point2<_>>::new();

        for line in self.lines_iter() {
            cdt.add_constraint_edge(
                Point2::new(line.start.x, line.start.y),
                Point2::new(line.end.x, line.end.y),
            )?;
        }

        let opt_bounding_rect = self.bounding_rect();
        let bounding_rect = if opt_bounding_rect.is_some() {
            opt_bounding_rect.unwrap()
        } else {
            return Err(InsertionError::NAN);
        };

        let mut y = bounding_rect.min().y + triangle_size / 2.0 * 3f32.sqrt();
        let mut x = bounding_rect.min().x + triangle_size;
        let mut alternate = false;

        while y < bounding_rect.max().y {
            while x < bounding_rect.max().x {
                if self.contains(&Coordinate { x, y }) {
                    cdt.insert(Point2::new(x, y))?;
                }
                x += triangle_size;
            }
            x = bounding_rect.min().x + triangle_size;
            if alternate {
                x += triangle_size / 2.0;
                alternate = false;
            } else {
                alternate = true;
            }
            y += triangle_size / 2.0 * 3f32.sqrt();
        }

        Ok(cdt)
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

fn vec2_to_u64(v: Vec2) -> u64 {
    let x = (f32::to_bits(v.x) as u64) << 32;
    let y = f32::to_bits(v.y) as u64;
    x + y
}

fn u64_to_vec2(u_64: u64) -> Vec2 {
    let x = f32::from_bits(u_64 as u32);
    let y = f32::from_bits((u_64 >> 32) as u32);
    Vec2::new(x, y)
}

fn coord_as_vec2(coord: &Coordinate<f32>) -> Vec2 {
    Vec2::new(coord.x, coord.y)
}

fn vec2_as_coord(v: &Vec2) -> Coordinate<f32> {
    Coordinate { x: v.x, y: v.y }
}
