use crate::*;
use bevy::{sprite::MaterialMesh2dBundle, utils::HashSet};
use cloth::Cloth;
use geo::*;
use image::GenericImageView;
use lyon::lyon_tessellation::{
    geometry_builder::simple_builder,
    math::{point, Point},
    path::Path,
    FillGeometryBuilder, FillOptions, FillTessellator, FillVertex, VertexBuffers,
};
use skeleton::Skeleton;
use spade::{ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation};
use std::cmp;
use std::{cmp::*, f32::consts::SQRT_2};
use std::{collections::HashMap, f32::consts::E};

const PIXEL_TO_UNIT_RATIO: f32 = 0.005;
pub const START_SCALE: f32 = 3.5;
pub const AVAILABLE_IMAGES: [&str; 7] = [
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
    pub queued_skins: Vec<AddSkinOrder>,
}

#[derive(Clone)]
pub enum AddSkinOrder {
    Grid {
        path: String,
        cols: u16,
        rows: u16,
        as_cloth: bool,
        cut_out: bool,
    },
    Delaunay {
        path: String,
        borderline_width: f32,
        triangle_size: f32,
    },
}

#[derive(Default)]
pub struct LineStrip {
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl LineStrip {
    fn simplify(&mut self, path: &str) {
        let img =
            image::open(format!("assets/{}", path)).expect(&format!("File not found!: {}", path));
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
    pub path: String,
    pub img: Image,
    pub vertices: Vec<Vec2>,
    pub edges: Vec<[usize; 2]>,
}
impl Contour {
    fn from_image(
        path: &str,
        asset_server: &AssetServer,
        image_assets: &Assets<Image>,
        borderline_width: u32,
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
        let (w, h) = (size.x as u32, size.y as u32);

        let offset = borderline_width + 1;
        let max_distance = borderline_width; // TODO: not working though it should ???
        let (distance_field_w, distance_field_h) = (w + offset * 2, h + offset as u32 * 2);

        let mut out = vec![vec![0; distance_field_h as usize]; distance_field_w as usize];

        // generate threshold distance field
        for output_x in 0..distance_field_w {
            for output_y in 0..distance_field_h {
                out[output_x as usize][distance_field_h as usize - output_y as usize - 1] =
                    if is_close_to_visible_pixel(output_x, output_y, img, offset, max_distance as f32) {
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
                if self.intersects_visible((a_vec2, b_vec2)) || end-start==max_edge_len as usize || i==line_strings[index].0.len() {
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

    fn to_multipoly(&self, max_edge_len: u32) -> MultiPolygon<f32> {
        let line_strings = self.to_simplified_line_strings(max_edge_len);
        let mut exteriors: Vec<usize> = vec![];
        let mut is_inside: Vec<Vec<usize>> = vec![vec![]; line_strings.len()];
        for i in 0..line_strings.len() {
            let poly = geo::Polygon::new(line_strings[i].clone(), vec![]);
            for j in 0..line_strings.len() {
                if i == j {
                    continue;
                } else {
                    if poly.contains(&line_strings[j]) {
                        is_inside[j].push(i);
                    }
                }
            }
        }
        for i in 0..is_inside.len() {
            if is_inside[i].len() == 0 {
                exteriors.push(i);
            }
        }
        let mut polygons: Vec<geo::Polygon<f32>> = exteriors
            .iter()
            .map(|&index| geo::Polygon::new(line_strings[index].clone(), vec![]))
            .collect();
        for i in 0..line_strings.len() {
            if is_inside[i].len() > 0 {
                polygons[0].interiors_push(line_strings[i].clone())
            }
        }
        MultiPolygon::new(polygons)
    }
}

trait DelaunayTriangulation {
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

        // let mut first = true;
        // let mut min = Vec2::new(0., 0.);
        // let mut max = Vec2::new(0., 0.);
        dbg!(self.coords_count());

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

        // let max_dist_squared = triangle_size.powi(2);
        // for x in (min.x as i32)..(max.x as i32) {
        //     for y in (min.y as i32)..(max.y as i32) {
        //         if !self.intersects(&Coordinate {
        //             x: x as f32,
        //             y: y as f32,
        //         }) {
        //             continue;
        //         }
        //         let mut add_vertex = true;
        //         let new_vertex = Point2::new(x as f32, y as f32);
        //         for v in cdt.vertices() {
        //             if v.position().distance_2(new_vertex) <= max_dist_squared {
        //                 add_vertex = false;
        //                 break;
        //             }
        //         }
        //         if add_vertex {
        //             cdt.insert(new_vertex)?;
        //         }
        //     }
        // }

        Ok(cdt)
    }
}

pub struct Polygon {
    pub path: String,
    pub line_strips: Vec<LineStrip>,
}
impl Polygon {
    fn to_geo_poly(&self) -> geo::Polygon<f32> {
        geo::Polygon::new(
            geo::LineString::new(
                self.line_strips[0]
                    .vertices
                    .iter()
                    .map(|v| Coordinate { x: v.x, y: v.y })
                    .collect(),
            ),
            vec![],
        )
    }
    fn to_geo_multipoly(&self) -> geo::MultiPolygon<f32> {
        let line_strings: Vec<LineString<f32>> = self
            .line_strips
            .iter()
            .map(|ls| {
                LineString(
                    ls.vertices
                        .iter()
                        .map(|v| Coordinate { x: v.x, y: v.y })
                        .collect(),
                )
            })
            .collect();
        let mut exteriors: Vec<usize> = vec![];
        let mut is_inside: Vec<Vec<usize>> = vec![vec![]; line_strings.len()];
        for i in 0..line_strings.len() {
            let poly = geo::Polygon::new(line_strings[i].clone(), vec![]);
            for j in 0..line_strings.len() {
                if i == j {
                    continue;
                } else {
                    if poly.contains(&line_strings[j]) {
                        is_inside[j].push(i);
                    }
                }
            }
        }
        for i in 0..is_inside.len() {
            if is_inside[i].len() == 0 {
                exteriors.push(i);
            }
        }
        let mut polygons: Vec<geo::Polygon<f32>> = exteriors
            .iter()
            .map(|&index| geo::Polygon::new(line_strings[index].clone(), vec![]))
            .collect();
        for i in 0..line_strings.len() {
            if is_inside[i].len() > 0 {
                polygons[0].interiors_push(line_strings[i].clone())
            }
        }
        MultiPolygon::new(polygons)
    }
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
            path: contour.path.clone(),
            line_strips: polygons,
        }
    }
    fn simplify(mut self) -> Self {
        for line_strip in self.line_strips.iter_mut() {
            line_strip.simplify(&self.path);
        }
        self
    }

    fn triangulate(
        &self,
        triangle_size: f32,
        asset_server: &AssetServer,
        image_assets: &Assets<Image>,
    ) -> Option<Skin> {
        let res_cdt = self.triangulate_delaunay(triangle_size);
        let cdt = if res_cdt.is_ok() {
            res_cdt.unwrap()
        } else {
            return None;
        };
        // = cdt
        // .vertices()
        // .map(|v| [v.position().x, v.position().y, 0.])
        // .collect();
        let mut vertices = vec![];
        let mut indices = vec![];

        let mut verts: HashMap<u64, u16> = HashMap::new();

        for face in cdt.inner_faces() {
            for v in face.vertices() {
                let key = vec2_to_u64(Vec2::new(v.position().x, v.position().y));
                let index: u16;
                if verts.contains_key(&key) {
                    index = verts[&key];
                } else {
                    index = verts.len() as u16;
                    verts.insert(key, index);
                    vertices.push([v.position().x, v.position().y, 0.]);
                }
                indices.push(index);
            }
        }

        indices.reverse();

        // let indices_len = indices.len();
        // for i in (0..indices_len).rev() {
        //     indices.push(indices[i]);
        // }

        // let uv_pixel = [cell_w * i as f32, cell_h * j as f32];
        // vertices.push([
        //     (uv_pixel[0] - w as f32 / 2.) * PIXEL_TO_UNIT_RATIO,
        //     (uv_pixel[1] - h as f32 / 2.) * PIXEL_TO_UNIT_RATIO,
        //     0.,
        // ]);
        let img_handle = asset_server.load(&self.path);
        let opt_img = image_assets.get(&img_handle);
        let dimensions = if opt_img.is_none() {
            println!("triangulate(): couldn't open image!");
            return None;
        } else {
            opt_img.unwrap().size()
        };

        // let img_handle = asset_server.load(&self.path);
        // let opt_img = image_assets.get(&img_handle);

        // if let Some(img) = opt_img {
        //     let size = img.size();
        //     let (w, h) = (size.x as u32, size.y as u32);
        //     for v in vertices {
        //         uvs.push([v[0], v[1]]);
        //     }
        // }

        let multipoly = self.to_geo_multipoly();

        for i in ((0..indices.len()).step_by(3)).rev() {
            let mut center = Vec2::new(0., 0.);
            for j in 0..3 {
                let v = vertices[indices[i + j] as usize];
                center.x += v[0];
                center.y += v[1];
                let v1 = vertices[indices[i + (j + 1) % 3] as usize];
                // if !geo_poly.intersects(&Line::new(Coordinate{x: v0[0], y: v0[1]},Coordinate{x: v1[0], y: v1[1]})) {
                //     remove = true;
                //     break;
                // }
            }
            center /= 3.;
            if !multipoly.intersects(&Coordinate {
                x: center.x,
                y: center.y,
            }) {
                for j in (0..3).rev() {
                    indices.swap_remove(i + j);
                }
            }
        }

        // Remove loose vertices
        let mut keep_vertex_indices: HashSet<usize> = HashSet::new();
        let mut removed_vertex_indices: Vec<usize> = vec![];
        for &ind in indices.iter() {
            keep_vertex_indices.insert(ind as usize);
        }
        for i in (0..vertices.len()).rev() {
            if !keep_vertex_indices.contains(&i) {
                vertices.remove(i);
                removed_vertex_indices.push(i);
            }
        }
        for ind in removed_vertex_indices {
            for index in indices.iter_mut() {
                if (ind as u16) < (*index) {
                    *index -= 1;
                }
            }
        }

        let mut uvs: Vec<[f32; 2]> = vec![];
        for v in vertices.iter() {
            let x = v[0] / dimensions.x;
            let y = 1. - v[1] / dimensions.y;
            uvs.push([x, y]);
        }

        for i in 0..vertices.len() {
            for j in 0..3 {
                vertices[i][j] *= PIXEL_TO_UNIT_RATIO;
            }
        }

        let skin = Skin {
            path: String::from(&self.path),
            vertices,
            uvs,
            indices,
            mesh_handle: None,
        };

        Some(skin)

        // for face in cdt.all_faces() {
        //     if let Some(f) = face.as_inner() {
        //         let mut v_first: Option<Vec2> = None;
        //         for vertex in f.vertices() {
        //             let v = Vec2::new(vertex.position().x as f32, vertex.position().y as f32);
        //             if v_first.is_none() {
        //                 path_builder.move_to(v);
        //                 v_first = Some(v);
        //             } else {
        //                 path_builder.line_to(v);
        //             }
        //         }
        //         path_builder.line_to(v_first.unwrap());
        //     }
        // }

        // let mut geometry = GeometryBuilder::build_as(
        //     &PathBuilder::new().build(),
        //     DrawMode::Stroke(StrokeMode::new(
        //         Color::Rgba {
        //             red: 1.,
        //             green: 1.,
        //             blue: 0.,
        //             alpha: 1.,
        //         },
        //         0.01,
        //     )),
        //     Transform::from_translation(Vec3::new(0., 0., 700.)),
        // );
    }

    fn triangulate_delaunay(
        &self,
        triangle_size: f32,
    ) -> Result<ConstrainedDelaunayTriangulation<Point2<f32>>, InsertionError> {
        let mut cdt = ConstrainedDelaunayTriangulation::<Point2<_>>::new();

        let mut first = true;
        let mut min = Vec2::new(0., 0.);
        let mut max = Vec2::new(0., 0.);

        for strip in self.line_strips.iter() {
            for edge in strip.edges.iter() {
                let p0 = strip.vertices[edge[0]];
                let p1 = strip.vertices[edge[1]];
                cdt.add_constraint_edge(Point2::new(p0.x, p0.y), Point2::new(p1.x, p1.y))?;
            }
            // Find min/max x and y
            for &v in strip.vertices.iter() {
                if first {
                    first = false;
                    min = v;
                    max = v;
                } else {
                    min.x = min.x.min(v.x);
                    min.y = min.y.min(v.y);
                    max.x = max.x.max(v.x);
                    max.y = max.y.max(v.y);
                }
            }
        }

        let dimensions = max - min;

        if dimensions.x == 0. || dimensions.y == 0. {
            dbg!("dimensions are invalid!");
            return Err(InsertionError::NAN);
        }

        let geo_poly = self.to_geo_poly();

        let max_dist_squared = triangle_size.powi(2);
        for x in (min.x as i32)..(max.x as i32) {
            for y in (min.y as i32)..(max.y as i32) {
                if !geo_poly.intersects(&Coordinate {
                    x: x as f32,
                    y: y as f32,
                }) {
                    continue;
                }
                let mut add_vertex = true;
                let new_vertex = Point2::new(x as f32, y as f32);
                for v in cdt.vertices() {
                    if v.position().distance_2(new_vertex) <= max_dist_squared {
                        add_vertex = false;
                        break;
                    }
                }
                if add_vertex {
                    cdt.insert(new_vertex)?;
                }
            }
        }

        // for i in 0..cols {
        //     for j in 0..rows {
        //         cdt.insert(Point2::new(
        //             min.x + i as f32 * dimensions.x / (cols as f32 - 1.),
        //             min.y + j as f32 * dimensions.y / (rows as f32 - 1.),
        //         ))?;
        //     }
        // }

        Ok(cdt)
    }

    fn triangulate_obsolete(&self) -> Skin {
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

        let img = image::open(format!("assets/{}", self.path)).expect("File not found!");
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
            path: self.path.clone(),
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
    pub vertices: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u16>,
    pub mesh_handle: Option<Mesh2dHandle>,
}
impl Skin {
    fn from_contour(contour: Contour, triangle_size: f32) -> Option<Skin> {
        let multipoly = contour.to_multipoly(triangle_size as u32);
        println!("from contour v_count: {}", multipoly.coords_count());
        let res_cdt = multipoly.triangulate_delaunay(triangle_size);
        let cdt = if res_cdt.is_ok() {
            res_cdt.unwrap()
        } else {
            return None;
        };

        let mut vertices = vec![];
        let mut indices = vec![];
        let mut unique_vertices: HashMap<u64, u16> = HashMap::new();

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

        indices.reverse();

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

        // Remove loose vertices
        let mut keep_vertex_indices: HashSet<usize> = HashSet::new();
        let mut removed_vertex_indices: Vec<usize> = vec![];
        for &ind in indices.iter() {
            keep_vertex_indices.insert(ind as usize);
        }
        for i in (0..vertices.len()).rev() {
            if !keep_vertex_indices.contains(&i) {
                vertices.remove(i);
                removed_vertex_indices.push(i);
            }
        }
        for ind in removed_vertex_indices {
            for index in indices.iter_mut() {
                if (ind as u16) < (*index) {
                    *index -= 1;
                }
            }
        }

        let (w, h) = (contour.img.size().x, contour.img.size().y);
        let mut uvs: Vec<[f32; 2]> = vec![];
        for v in vertices.iter() {
            let x = v[0] / w;
            let y = 1. - v[1] / h;
            uvs.push([x, y]);
        }

        for i in 0..vertices.len() {
            for j in 0..3 {
                vertices[i][j] *= PIXEL_TO_UNIT_RATIO;
            }
        }

        let skin = Skin {
            path: String::from(contour.path),
            vertices,
            uvs,
            indices,
            mesh_handle: None,
        };

        Some(skin)
    }
    pub fn gl_vertices(&self, gl_transform: &GlobalTransform) -> Vec<[f32; 3]> {
        let (gl_scale, gl_rotation, gl_translation) = gl_transform.to_scale_rotation_translation();
        self.vertices
            .iter()
            .map(|v| {
                let mut res = Vec3::from_slice(v);
                res *= gl_scale;
                res = Quat::mul_vec3(gl_rotation, res);
                res += gl_translation;
                [res.x, res.y, 0.]
            })
            .collect::<Vec<[f32; 3]>>()
    }
    pub fn grid_mesh(
        path: &str,
        asset_server: &AssetServer,
        image_assets: &Assets<Image>,
        cols: u16,
        rows: u16,
        cut_out: bool,
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
                vertices,
                uvs,
                indices,
                mesh_handle: None,
            };
            // // Remove reduntant vertices and corresponding uvs and indices
            if cut_out {
                for i in (0..skin.uvs.len()).rev() {
                    let v = skin.uvs[i];
                    let coord = [
                        min((v[0] * w as f32) as u32, w - 1),
                        min((v[1] * h as f32) as u32, h - 1),
                    ];
                    // if uv is out of image or pixel at uv is transparent remove
                    if !is_close_to_visible_pixel(
                        coord[0],
                        coord[1],
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

// For some reason doesn't work with max_dist 1.0
fn is_close_to_visible_pixel(output_x: u32, output_y: u32, img: &Image, offset: u32, max_dist: f32) -> bool {
    let max_dist_ceil = f32::ceil(max_dist) as i32;
    let x: i32 = output_x as i32 - offset as i32;
    let y: i32 = output_y as i32 - offset as i32;
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

pub fn system_set() -> SystemSet {
    SystemSet::new().with_system(add_skins)
}

pub fn add_pooh_on_startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut state: ResMut<State>,
    asset_server: Res<AssetServer>,
    image_assets: Res<Assets<Image>>,
) {
    state.queued_skins.push(AddSkinOrder::Grid {
        path: String::from("img/honey.png"),
        cols: 6,
        rows: 10,
        as_cloth: true,
        cut_out: false,
    });
    state.queued_skins.push(AddSkinOrder::Grid {
        path: String::from("img/pooh.png"),
        cols: 30,
        rows: 30,
        as_cloth: false,
        cut_out: true,
    });
}

fn add_skin(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
    asset_server: &AssetServer,
    order: &AddSkinOrder,
    image_assets: &Assets<Image>,
) -> Option<(Entity, Mesh2dHandle)> {
    let opt_skin: Option<Skin> = match order {
        AddSkinOrder::Grid {
            path,
            cols,
            rows,
            as_cloth: _,
            cut_out,
        } => Skin::grid_mesh(path, asset_server, image_assets, *cols, *rows, *cut_out),
        AddSkinOrder::Delaunay {
            path,
            borderline_width,
            triangle_size,
        } => {
            let contour =
                Contour::from_image(path, asset_server, image_assets, *borderline_width as u32)?;
            Skin::from_contour(contour, *triangle_size)
        }
    };

    if opt_skin.is_none() {
        dbg!("couldn't generate skin");
        return None;
    }

    let mut skin = opt_skin.unwrap();

    let vertices = skin
        .vertices
        .clone()
        .iter()
        .map(|v| [v[0], v[1], 0.])
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

    if let AddSkinOrder::Grid {
        path: _,
        cols,
        rows,
        as_cloth,
        cut_out: _,
    } = order
    {
        if *as_cloth {
            let bounding_box = meshes.get(&handle.0).unwrap().compute_aabb().unwrap();
            let diagonal = (bounding_box.max() - bounding_box.min()) * skin::START_SCALE;
            let cloth = Cloth::new(
                Vec3::new(0., 0., 0.),
                diagonal.x,
                diagonal.y,
                *cols as usize,
                *rows as usize,
            )
            .with_stiffness(10);
            commands.entity(skin_id).insert(cloth);
        }
    }

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
        if add_skin(
            &mut commands,
            &mut meshes,
            &mut materials,
            &asset_server,
            &event,
            &image_assets,
        )
        .is_some()
        {
            state.queued_skins.swap_remove(i);
        }
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
