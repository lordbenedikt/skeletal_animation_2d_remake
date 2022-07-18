use crate::*;
use skin::Skin;

#[derive(Component, Default)]
pub struct Cloth {
    point_masses: Vec<PointMass>,
    links: Vec<Link>,
    stiffness: u32,
    is_tearable: bool,
}
impl Cloth {
    pub fn from_mesh(mesh_handle: Mesh2dHandle, meshes: &Assets<Mesh>) -> Self {
        let mesh = meshes.get(mesh_handle.0).unwrap();
        let mut vertices = mesh::get_vertices(mesh);
        for v in vertices.iter_mut() {
            *v *= Vec3::new(3.5,3.5,1.);
        }
        let mut point_masses = vec![];
        for i in 0..vertices.len() {
            point_masses.push(PointMass::new(vertices[i], i <= 20));
        }
        let links: Vec<Link> = vec![];
        let mut cloth =         Self {
            point_masses,
            links,
            stiffness: 4,
            is_tearable: false,
        };
        if let Indices::U16(indices) = mesh.indices().unwrap() {
            for i in (0..indices.len()).step_by(3) {
                let mut already_present = false;
                for j in 0..3 {
                    let ind_0 = i + j;
                    let ind_1 = i + ((j + 1) % 3);
                    for link in cloth.links.iter() {
                        if indices[ind_0] == link.indices[0] as u16
                            || indices[ind_0] == link.indices[1] as u16
                        {
                            if indices[ind_1] == link.indices[0] as u16
                                || indices[ind_1] == link.indices[1] as u16
                            {
                                already_present = true;
                                break;
                            }
                        }
                        let pos_0 = vertices[indices[ind_0] as usize];
                        let pos_1 = vertices[indices[ind_1] as usize];
                        if pos_0.x != pos_1.x && pos_0.y != pos_1.y {
                            already_present = true;
                            break;
                        }
                    }
                    if !already_present {
                        cloth.add_link([indices[ind_0] as usize, indices[ind_1] as usize]);
                    }
                }
            }
        }
        cloth

    }
    pub fn new(pos: Vec3, w: f32, h: f32, cols: usize, rows: usize) -> Self {
        let mut cloth = Cloth::default().with_stiffness(4);
        let cell_w = w / cols as f32;
        let cell_h = h / rows as f32;
        for row in 0..=rows {
            for col in 0..=cols {
                let current_pos = pos + Vec3::new(cell_w * col as f32, -cell_h * row as f32, 0.);
                let pm = PointMass::new(current_pos, row == 0);
                cloth.point_masses.push(pm);
            }
        }
        for row in 0..rows {
            for col in 0..cols {
                let i0 = row * (cols + 1) + col; // top left
                let i1 = i0 + 1; // top right
                let i2 = i1 + cols + 1; // bottom right
                let i3 = i2 - 1; // bottom left
                cloth.add_link([i0, i1]);
                cloth.add_link([i0, i3]);
                if col == cols - 1 {
                    cloth.add_link([i1, i2]);
                }
                if row == rows - 1 {
                    cloth.add_link([i2, i3]);
                }
            }
        }
        cloth
    }
    fn add_link(&mut self, indices: [usize; 2]) {
        self.links.push(Link::new(&self, indices));
    }
    // Move point masses so that the distance between them equals resting distance
    fn solve(&mut self) {
        for _ in 0..self.stiffness {
            for i in (0..self.links.len()).rev() {
                let link = &self.links[i];
                let pm0 = &self.point_masses[link.indices[0]];
                let pm1 = &self.point_masses[link.indices[1]];

                // Do nothing if both point masses are pinned
                if pm0.pin.is_some() && pm1.pin.is_some() {
                    continue;
                }

                let diff = pm1.position - pm0.position;
                if self.is_tearable && diff.length() > link.tear_distance {
                    self.links.swap_remove(i);
                } else {
                    let correction_amount = link.resting_distance - diff.length();

                    // If one point mass is point, only move the other, else distribute correction
                    let mass_distribution = if pm0.pin.is_some() {
                        0.
                    } else if pm1.pin.is_some() {
                        1.
                    } else {
                        pm1.mass / (pm0.mass + pm1.mass)
                    };

                    let correction = diff.normalize() * correction_amount;
                    self.point_masses[link.indices[0]].position -= correction * mass_distribution;
                    self.point_masses[link.indices[1]].position +=
                        correction * (1. - mass_distribution);
                }
            }
        }
    }
    fn update(&mut self) {
        for point_mass in self.point_masses.iter_mut() {
            point_mass.update();
        }
        self.solve();
    }
    fn draw_debug_lines(&self, debug_drawer: &mut DebugDrawer) {
        for link in self.links.iter() {
            let start = self.point_masses[link.indices[0]].position;
            let end = self.point_masses[link.indices[1]].position;
            debug_drawer.line(start.truncate(), end.truncate(), COLOR_DEFAULT);
        }
    }
    fn with_stiffness(mut self, stiffness: u32) -> Self {
        self.stiffness = stiffness;
        self
    }
    pub fn vertex_is_free(&self, index: usize) -> bool {
        self.point_masses[index].pin.is_none()
    }
}

#[derive(Default)]
pub struct PointMass {
    position: Vec3,
    last_position: Vec3,
    velocity: Vec3,
    acceleration: Vec3,
    mass: f32,
    pin: Option<Vec3>,
}
impl PointMass {
    pub fn new(position: Vec3, is_pinned: bool) -> Self {
        PointMass {
            position,
            last_position: position,
            pin: if is_pinned { Some(position) } else { None },
            mass: 1.,
            ..Default::default()
        }
    }
    pub fn update(&mut self) {
        // If mass is pinned reset position to pin
        if let Some(pin) = self.pin {
            return;
        }

        // Compute velocity from current position - last position
        self.velocity = self.position - self.last_position;

        // Save position for next frame;
        self.last_position = self.position;

        // Find next position
        self.position = self.position + self.velocity + self.acceleration;

        // Set acceleration to gravity
        self.acceleration = Vec3::new(0., -0.01, 0.);
    }
}

pub struct Link {
    resting_distance: f32,
    tear_distance: f32,
    indices: [usize; 2],
}
impl Link {
    fn new(cloth: &Cloth, indices: [usize; 2]) -> Self {
        let resting_distance = Vec3::distance(
            cloth.point_masses[indices[0]].position,
            cloth.point_masses[indices[1]].position,
        );
        Link {
            resting_distance,
            tear_distance: 5. * resting_distance,
            indices,
        }
    }
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(update_cloth)
        .with_system(apply_mesh_to_cloth)
}

pub fn create_cloth(mut commands: Commands) {
    // let skin = Skin::grid_mesh("Unbenannt.png", 10, 10);
    // let cloth = Cloth::new(Vec2::new(0., 0.), 5., 4., 10, 10);

    // commands.spawn().insert(cloth).insert(skin);
}

pub fn update_cloth(
    meshes: Res<Assets<Mesh>>,
    mut q: Query<(&mut Cloth, &Skin)>,
    mut debug_drawer: ResMut<DebugDrawer>,
) {
    for (mut cloth, skin) in q.iter_mut() {
        // apply pinned point masses to skeleton (get positions from mesh, updated in skeleton.rs)
        let mesh = meshes.get(skin.mesh_handle.clone().unwrap().0).unwrap();
        for i in 0..cloth.point_masses.len() {
            // if point mass isn't pinned continue with next point mass
            if cloth.point_masses[i].pin.is_none() {
                continue;
            }

            let pos = mesh::get_vertex(mesh, i);
            cloth.point_masses[i].position = Vec3::from_slice(&pos);
        }

        cloth.update();
        let pm = &mut cloth.point_masses[110];
    }
}

pub fn apply_mesh_to_cloth(mut meshes: ResMut<Assets<Mesh>>, q: Query<(&Cloth, &Skin)>) {
    for (cloth, skin) in q.iter() {
        let mesh = meshes.get_mut(skin.mesh_handle.clone().unwrap().0).unwrap();

        // update mesh vertices
        let mut vertices: Vec<[f32; 3]> = vec![];
        for i in 0..cloth.point_masses.len() {
            let pm = &cloth.point_masses[i];
            if pm.pin.is_some() {
                let v = mesh::get_vertex(&mesh, i);
                vertices.push([v[0], v[1], skin.depth]);
            } else {
                vertices.push([pm.position[0], pm.position[1], pm.position[2]]);
            }
        }
        let mut colors: Vec<u32> = vec![];
        for i in 0..cloth.point_masses.len() {
            let pm = &cloth.point_masses[i];
            let lightest = 0.;
            let color = pm.position.z;
            let r = 255;
            colors.push(Color::RED.as_linear_rgba_u32());
        }
        // mesh.insert_attribute(
        //     MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Uint32),
        //     v_color,
        // );

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    }
}
