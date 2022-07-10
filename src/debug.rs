use crate::*;

#[derive(Default)]
pub struct DebugDrawer {
    lines: Vec<Line>,
    squares: Vec<Square>,
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
}

pub fn draw_debug_shapes(debug_drawer: Res<DebugDrawer>, mut lines: ResMut<DebugLines>) {
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
}

pub fn clear_debug_drawer(mut debug_drawer: ResMut<DebugDrawer>) {
    debug_drawer.clear();
}
