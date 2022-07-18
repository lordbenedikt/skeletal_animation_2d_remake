use crate::*;

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
        self.lines.push(Line { start, end, color })
    }
    pub fn square(&mut self, center: Vec2, s: i32, color: Color) {
        self.squares.push(Square { center, s, color })
    }
    pub fn clear(&mut self) {
        self.lines.clear();
        self.squares.clear();
    }
    pub fn line_permanent(&mut self, start: Vec2, end: Vec2, color: Color) {
        self.lines_permanent.push(Line { start, end, color })
    }
    pub fn square_permanent(&mut self, center: Vec2, s: i32, color: Color) {
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

pub struct Line {
    start: Vec2,
    end: Vec2,
    color: Color,
}

pub struct Square {
    center: Vec2,
    s: i32,
    color: Color,
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(draw_skin_mesh)
        .with_system(draw_bones.after(draw_skin_mesh))
        .with_system(draw_debug_shapes.after(draw_bones))
        .with_system(clear_debug_drawer.after(draw_debug_shapes))
        .with_system(enable_debug_lines)
}

fn draw_square(lines: &mut DebugLines, square: &Square) {
    let scalar = 1. / PIXELS_PER_UNIT as f32;
    let s = scalar * square.s as f32;
    let half_s = s / 2.;
    let mut xx = -half_s;
    loop {
        if xx > half_s {
            break;
        }
        let x = square.center.x + xx;
        let y0 = square.center.y - half_s;
        let y1 = square.center.y + half_s;
        let start = Vec3::new(x, y0, 0.);
        let end = Vec3::new(x, y1, 0.);
        lines.line_colored(start, end, 0., square.color);
        xx += scalar / 2.;
    }
    for i in 0..2 {
        let x0 = square.center.x - half_s;
        let x1 = square.center.x + half_s;
        let y = square.center.y + if i == 0 { half_s } else { -half_s };
        let start = Vec3::new(x0, y, 0.);
        let end = Vec3::new(x1, y, 0.);
        lines.line_colored(start, end, 0., square.color);
    }
}

pub fn draw_debug_shapes(debug_drawer: Res<DebugDrawer>, mut lines: ResMut<DebugLines>) {
    // draw for one frame
    for line in debug_drawer.lines.iter() {
        lines.line_colored(line.start.extend(0.), line.end.extend(0.), 0., line.color);
    }
    for square in debug_drawer.squares.iter() {
        draw_square(&mut lines, square);
    }
    // draw every frame
    for line in debug_drawer.lines_permanent.iter() {
        lines.line_colored(line.start.extend(0.), line.end.extend(0.), 0., line.color);
    }
    for square in debug_drawer.squares_permanent.iter() {
        let scalar = 0.01;
        for i in 0..square.s {
            let start = Vec3::new(
                square.center.x - scalar * (square.s / 2 + i) as f32,
                square.center.y - scalar * (square.s / 2) as f32,
                0.,
            );
            let end = Vec3::new(
                square.center.x - scalar * (square.s / 2 + i) as f32,
                square.center.y + scalar * (square.s / 2) as f32,
                0.,
            );
            lines.line_colored(start, end, 0., square.color)
        }
    }
}

pub fn clear_debug_drawer(mut debug_drawer: ResMut<DebugDrawer>) {
    debug_drawer.clear();
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
        let mesh = meshes.get(skin.mesh_handle.clone().unwrap().0).unwrap();
        let vertices: Vec<Vec3> = mesh::get_vertices(mesh);

        let color = if transformable.is_selected {
            COLOR_SELECTED
        } else {
            COLOR_DEFAULT
        };

        // draw VERTICES
        for i in 0..vertices.len() {
            debug_drawer.square(vertices[i].truncate(), 3, color);
        }

        // draw LINES
        let mut i = 2;
        while i < skin.indices.len() {
            let p0 = vertices[skin.indices[i] as usize].truncate();
            let p1 = vertices[skin.indices[i - 1] as usize].truncate();
            let p2 = vertices[skin.indices[i - 2] as usize].truncate();
            debug_drawer.line(p0, p1, color);
            debug_drawer.line(p1, p2, color);
            debug_drawer.line(p2, p0, color);
            i += 3;
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
        let z = 0.001;
        let scale = gl_transform.scale;
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
            points[i].x *= scale.x;
            points[i].y *= scale.y;
        }
        for i in 0..points.len() {
            debug_drawer.line(
                (gl_transform.translation + Quat::mul_vec3(gl_transform.rotation, points[i]))
                    .truncate(),
                (gl_transform.translation
                    + Quat::mul_vec3(gl_transform.rotation, points[(i + 1) % points.len()]))
                .truncate(),
                color,
            );
        }
        debug_drawer.square(gl_transform.translation.truncate(), 7, color);
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
