use bevy::utils::{HashMap, HashSet};

use crate::*;

const RIGHT_HALF_BITMASK: u32 = (1 << 16) - 1;

pub struct DebugDrawer {
    lines: Vec<Line>,
    squares: Vec<Square>,
    lines_permanent: Vec<Line>,
    squares_permanent: Vec<Square>,
    pub bone_debug_enabled: bool,
    pub mesh_debug_enabled: bool,
}
impl DebugDrawer {
    pub fn line(&mut self, start: Vec2, end: Vec2, color: Color) {
        self.lines.push(Line {
            start,
            end,
            color,
            weight: 1f32,
        })
    }
    pub fn line_thick(&mut self, start: Vec2, end: Vec2, color: Color, weight: f32) {
        self.lines.push(Line {
            start,
            end,
            color,
            weight,
        })
    }
    pub fn square(&mut self, center: Vec2, s: f32, color: Color) {
        self.squares.push(Square { center, s, color })
    }
    pub fn clear(&mut self) {
        self.lines.clear();
        self.squares.clear();
    }
    pub fn line_permanent(&mut self, start: Vec2, end: Vec2, color: Color) {
        self.lines_permanent.push(Line {
            start,
            end,
            color,
            weight: 1f32,
        })
    }
    pub fn square_permanent(&mut self, center: Vec2, s: f32, color: Color) {
        self.squares_permanent.push(Square { center, s, color })
    }
    pub fn clear_permanent(&mut self) {
        self.lines_permanent.clear();
        self.squares_permanent.clear();
    }
}
impl Default for DebugDrawer {
    fn default() -> Self {
        Self {
            lines: vec![],
            squares: vec![],
            lines_permanent: vec![],
            squares_permanent: vec![],
            bone_debug_enabled: true,
            mesh_debug_enabled: false,
        }
    }
}

const SCALAR: f32 = 1. / PIXELS_PER_UNIT as f32;

#[derive(Clone)]
pub struct Line {
    start: Vec2,
    end: Vec2,
    color: Color,
    weight: f32,
}

#[derive(Clone)]
pub struct Square {
    center: Vec2,
    s: f32,
    color: Color,
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(draw_skin_bounding_box.before(draw_skin_mesh))
        .with_system(draw_skin_mesh)
        .with_system(draw_ccd_target)
        .with_system(draw_bones.after(draw_skin_mesh).after(draw_ccd_target))
        .with_system(draw_permanent_debug_shapes.before(draw_debug_shapes))
        .with_system(draw_debug_shapes.after(draw_bones))
        .with_system(clear_debug_drawer.after(draw_debug_shapes))
        .with_system(enable_debug_lines)
}

fn draw_line_thick(line: &Line, lines: &mut DebugLines) {
    let diff = (line.end - line.start).extend(0.);
    let right =
        Quat::mul_vec3(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), diff).normalize();
    let mut offset = -line.weight / 2.;
    loop {
        if offset > line.weight / 2. {
            break;
        }
        lines.line_colored(
            line.start.extend(0.) + offset * right * SCALAR,
            line.end.extend(0.) + offset * right * SCALAR,
            0.,
            line.color,
        );
        offset += 0.5;
    }
}

fn draw_square(square: &Square, lines: &mut DebugLines) {
    let frac_s_2_scaled = square.s as f32 / 2. * SCALAR;
    draw_line_thick(
        &Line {
            start: square.center - Vec2::new(frac_s_2_scaled, 0.),
            end: square.center + Vec2::new(frac_s_2_scaled, 0.),
            color: square.color,
            weight: square.s,
        },
        lines,
    );
    draw_line_thick(
        &Line {
            start: square.center - Vec2::new(0., frac_s_2_scaled),
            end: square.center + Vec2::new(0., frac_s_2_scaled),
            color: square.color,
            weight: square.s,
        },
        lines,
    );
}

pub fn draw_debug_shapes(mut debug_drawer: ResMut<DebugDrawer>, mut lines: ResMut<DebugLines>) {
    let scalar = 0.01;
    // draw for one frame
    for line in debug_drawer.lines.iter() {
        if line.weight == 1f32 {
            lines.line_colored(line.start.extend(0.), line.end.extend(0.), 0., line.color);
        } else {
            draw_line_thick(line, &mut lines);
        }
    }
    for i in 0..debug_drawer.squares.len() {
        let square = &debug_drawer.squares[i].clone();
        draw_square(square, &mut lines);
    }
}

pub fn draw_permanent_debug_shapes(mut debug_drawer: ResMut<DebugDrawer>) {
    for i in 0..debug_drawer.squares_permanent.len() {
        let line = debug_drawer.lines[i].clone();
        debug_drawer.lines.push(line);
    }
    for i in 0..debug_drawer.squares_permanent.len() {
        let square = debug_drawer.squares[i].clone();
        debug_drawer.squares.push(square);
    }
}

pub fn clear_debug_drawer(mut debug_drawer: ResMut<DebugDrawer>) {
    debug_drawer.clear();
}

pub fn draw_skin_bounding_box(
    meshes: Res<Assets<Mesh>>,
    mut q: Query<(&mut Transformable, &skin::Skin)>,
    mut debug_drawer: ResMut<DebugDrawer>,
) {
    for (mut transformable, skin) in q.iter_mut() {
        // if mesh doesn't exist, continue
        let opt_mesh = meshes.get(&skin.mesh_handle.clone().unwrap().0);
        if opt_mesh.is_none() {
            continue;
        }
        let mesh = opt_mesh.unwrap();

        let vertices: Vec<Vec3> = mesh::get_vertices(mesh);

        let mut sum = Vec3::ZERO;
        let mut min = mesh::get_vertex(mesh, 0);
        let mut max = mesh::get_vertex(mesh, 0);

        for i in 0..vertices.len() {
            let vertex = mesh::get_vertex(mesh, i);
            sum += Vec3::from_slice(&vertex);
            for index in 0..2 {
                min[index] = f32::min(min[index], vertex[index]);
                max[index] = f32::max(max[index], vertex[index]);
            }
        }

        let average = sum / vertices.len() as f32;
        transformable.collision_shape =
            Shape::Rectangle(Vec2::from_slice(&min), Vec2::from_slice(&max));

        let color = if transformable.is_selected {
            COLOR_SELECTED
        } else {
            COLOR_DEFAULT
        };

        // if skin isn't selected, don't draw bounding box
        if !transformable.is_selected {
            continue;
        }
        // draw bounding box
        debug_drawer.line_thick(
            Vec2::new(min[0], min[1]),
            Vec2::new(min[0], max[1]),
            color,
            3.,
        );
        debug_drawer.line_thick(
            Vec2::new(max[0], min[1]),
            Vec2::new(max[0], max[1]),
            color,
            3.,
        );
        debug_drawer.line_thick(
            Vec2::new(min[0], min[1]),
            Vec2::new(max[0], min[1]),
            color,
            3.,
        );
        debug_drawer.line_thick(
            Vec2::new(min[0], max[1]),
            Vec2::new(max[0], max[1]),
            color,
            3.,
        );
    }
}

pub fn draw_skin_mesh(
    meshes: Res<Assets<Mesh>>,
    q: Query<(&Transformable, &skin::Skin)>,
    mut debug_drawer: ResMut<DebugDrawer>,
) {
    if !debug_drawer.mesh_debug_enabled {
        return;
    };

    for (transformable, skin) in q.iter() {
        let opt_mesh = meshes.get(&skin.mesh_handle.clone().unwrap().0);
        if opt_mesh.is_none() {
            continue;
        }
        let mesh = opt_mesh.unwrap();

        let vertices: Vec<Vec3> = mesh::get_vertices(mesh);

        let color = if transformable.is_selected {
            COLOR_SELECTED
        } else {
            COLOR_DEFAULT
        };

        // draw VERTICES
        for i in 0..vertices.len() {
            debug_drawer.square(vertices[i].truncate(), 5., color);
        }

        // draw LINES
        let mut i = 2;
        let mut lines_hashset: HashSet<u32> = HashSet::new();
        while i < skin.indices.len() {
            let inds = [
                skin.indices[i] as usize,
                skin.indices[i - 1] as usize,
                skin.indices[i - 2] as usize,
            ];
            // Add each unique combination of indices to lines_hashset
            for j in 0..inds.len() {
                let mut ii = [inds[j] as u32, inds[(j + 1) % inds.len()] as u32];
                ii.sort_unstable();
                // Store both indices as a single u32
                lines_hashset.insert((ii[0] << 16) + ii[1]);
            }
            i += 3;
        }
        for line in lines_hashset {
            debug_drawer.line_thick(
                vertices[(line >> 16) as usize].truncate(), // 16 most significant bits
                vertices[(line & RIGHT_HALF_BITMASK) as usize].truncate(), // 16 least significant bits
                color,
                1.,
            )
        }
    }
}

pub fn draw_bones(
    mut debug_drawer: ResMut<DebugDrawer>,
    bone_gl_transforms: Query<(&GlobalTransform, &bone::Bone, &Transformable)>,
) {
    if !debug_drawer.bone_debug_enabled {
        return;
    };

    for (gl_transform, _, transformable) in bone_gl_transforms.iter() {
        let (gl_scale, gl_rotation,gl_translation) = gl_transform.to_scale_rotation_translation();
        let z = 0.001;
        let color = if transformable.is_selected {
            COLOR_SELECTED
        } else {
            COLOR_DEFAULT
        };
        let mut points = vec![
            Vec3::new(0., 0., z),
            Vec3::new(-0.1, 0.1, z),
            Vec3::new(0., 1., z),
            Vec3::new(0.1, 0.1, z),
            Vec3::new(0., 0., z),
        ];
        for i in 0..points.len() {
            points[i].x *= gl_scale.x;
            points[i].y *= gl_scale.y;
        }
        for i in 0..points.len() {
            debug_drawer.line_thick(
                (gl_translation + Quat::mul_vec3(gl_rotation, points[i]))
                    .truncate(),
                (gl_translation
                    + Quat::mul_vec3(gl_rotation, points[(i + 1) % points.len()]))
                .truncate(),
                color,
                3.,
            );
        }
        debug_drawer.square(gl_translation.truncate(), 7., color);
    }
}

pub fn enable_debug_lines(keys: Res<Input<KeyCode>>, mut debug_drawer: ResMut<DebugDrawer>) {
    if keys.just_pressed(KeyCode::D) {
        debug_drawer.bone_debug_enabled = !debug_drawer.bone_debug_enabled;
    }
    if keys.just_pressed(KeyCode::M) {
        debug_drawer.mesh_debug_enabled = !debug_drawer.mesh_debug_enabled;
    }
}

pub fn draw_ccd_target(
    debug_drawer: Res<DebugDrawer>,
    mut q: Query<(&Transformable, &mut Visibility, &mut Sprite), With<ccd::Target>>,
) {
    for (transformable, mut visibility, mut sprite) in q.iter_mut() {
        if debug_drawer.bone_debug_enabled {
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
            continue;
        }
        if transformable.is_selected {
            sprite.color = COLOR_SELECTED;
        } else {
            sprite.color = COLOR_DEFAULT;
        }
    }
}
