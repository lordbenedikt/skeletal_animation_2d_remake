use crate::*;

#[derive(Default)]
pub struct DebugDrawer {
    lines: Vec<Line>,
    squares: Vec<Square>,
    lines_permanent: Vec<Line>,
    squares_permanent: Vec<Square>,
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
        .with_system(draw_debug_shapes)
        .with_system(clear_debug_drawer.after(draw_debug_shapes))
        .with_system(draw_mesh)
}

pub fn draw_debug_shapes(debug_drawer: Res<DebugDrawer>, mut lines: ResMut<DebugLines>) {
    // draw for one frame
    for line in debug_drawer.lines.iter() {
        lines.line_colored(line.start.extend(0.), line.end.extend(0.), 0., line.color);
    }
    for square in debug_drawer.squares.iter() {
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

fn draw_mesh(q: Query<(&GlobalTransform, &Transformable, &skin::Skin)>, mut debug_drawer: ResMut<DebugDrawer>) {
    for (gl_transform, transformable, skin) in q.iter() {
        if !transformable.is_selected {
            continue;
        }
        // for vertex in skin.vertices.iter() {
        //     debug_drawer.square(Vec2::from_slice(vertex), 8, COLOR_DEFAULT);
        // }
        let mut i = 2;
        let gl_vertices = skin.gl_vertices(gl_transform);
        while i < skin.indices.len() {
            let p0 = Vec2::from_slice(&gl_vertices[skin.indices[i] as usize]);
            let p1 = Vec2::from_slice(&gl_vertices[skin.indices[i - 1] as usize]);
            let p2 = Vec2::from_slice(&gl_vertices[skin.indices[i - 2] as usize]);
            debug_drawer.line(p0, p1, COLOR_DEFAULT);
            debug_drawer.line(p1, p2, COLOR_DEFAULT);
            debug_drawer.line(p2, p0, COLOR_DEFAULT);
            i += 3;
        }
    }
}
