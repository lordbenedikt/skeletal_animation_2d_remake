use crate::{image::Pixels, mesh_gen::Contour, *};
use bevy::{sprite::MaterialMesh2dBundle, utils::HashSet};
use cloth::Cloth;
use geo::*;
use lyon::lyon_tessellation::{
    geometry_builder::simple_builder,
    math::{point, Point},
    path::Path,
    FillOptions, FillTessellator, VertexBuffers,
};
use spade::{ConstrainedDelaunayTriangulation, InsertionError, Point2, Triangulation};
use std::collections::HashMap;
use std::{cmp::*, f32::consts::SQRT_2};

pub const PIXEL_TO_UNIT_RATIO: f32 = 0.005;
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
        let (vertices,uvs,indices) = contour.to_mesh(triangle_size);

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
            let (w, h) = (size.x as i32, size.y as i32);

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
                        min((v[0] * w as f32) as i32, w - 1),
                        min((v[1] * h as f32) as i32, h - 1),
                    ];
                    // if uv is out of image or pixel at uv is transparent remove
                    if !img.is_close_to_visible(
                        coord[0],
                        coord[1],
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
                Contour::from_image(path, asset_server, image_assets, *borderline_width as i32)?;
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

pub fn system_set() -> SystemSet {
    SystemSet::new().with_system(add_skins)
}