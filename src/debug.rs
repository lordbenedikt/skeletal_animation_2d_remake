use bevy::utils::HashSet;

use crate::*;

const RIGHT_HALF_BITMASK: u32 = (1 << 16) - 1;

#[derive(Resource)]
pub struct DebugDrawer {
    lines: Vec<Line>,
    squares: Vec<Square>,
    lines_permanent: Vec<Line>,
    squares_permanent: Vec<Square>,
    pub bone_debug_enabled: bool,
    pub mesh_debug_enabled: bool,
    pub test_entities: Vec<Entity>,
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
            test_entities: vec![],
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
        .with_system(draw_skin_bounding_box.before(draw_all_debug_shapes))
        .with_system(draw_skin_mesh.before(draw_all_debug_shapes))
        .with_system(draw_select_box.before(draw_all_debug_shapes))
        .with_system(draw_ccd_target)
        .with_system(
            draw_bones
                .after(draw_skin_mesh)
                .after(draw_ccd_target)
                .before(draw_all_debug_shapes),
        )
        .with_system(draw_permanent_debug_shapes)
        .with_system(draw_all_debug_shapes.after(draw_bones))
        .with_system(clear_debug_drawer.after(draw_all_debug_shapes))
        .with_system(enable_debug_lines)
}

// fn draw_line_thick(line: &Line, lines: &mut DebugLines) {
//     let diff = (line.end - line.start).extend(0.);
//     let right =
//         Quat::mul_vec3(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2), diff).normalize();
//     let mut offset = -line.weight / 2.;
//     loop {
//         if offset > line.weight / 2. {
//             break;
//         }
//         lines.line_colored(
//             line.start.extend(0.) + offset * right * SCALAR,
//             line.end.extend(0.) + offset * right * SCALAR,
//             0.,
//             line.color,
//         );
//         offset += 0.5;
//     }
// }

fn draw_square(square: &Square, debug_drawer: &mut DebugDrawer) {
    let frac_s_2_scaled = square.s as f32 / 2. * SCALAR;
    debug_drawer.line_thick(
        square.center - Vec2::new(frac_s_2_scaled, 0.),
        square.center + Vec2::new(frac_s_2_scaled, 0.),
        square.color,
        square.s,
    );
    // debug_drawer.line_thick(
    //     square.center - Vec2::new(0., frac_s_2_scaled),
    //     square.center + Vec2::new(0., frac_s_2_scaled),
    //     square.color,
    //     square.s,
    // );
}

pub fn draw_all_debug_shapes(
    mut debug_drawer: ResMut<DebugDrawer>,
    mut commands: Commands,
    q: Query<Entity>,
) {
    for i in (0..debug_drawer.test_entities.len()).rev() {
        let entity = debug_drawer.test_entities[i];
        if q.get(entity).is_ok() {
            debug_drawer.test_entities.swap_remove(i);
            commands.entity(entity).despawn();
        }
    }
    for i in 0..debug_drawer.squares.len() {
        let square = &debug_drawer.squares[i].clone();
        draw_square(square, &mut debug_drawer);
    }

    let scalar = 1. / PIXELS_PER_UNIT as f32;
    while !debug_drawer.lines.is_empty() {
        let mut path_builder = PathBuilder::new();
        let first_line = debug_drawer.lines[debug_drawer.lines.len() - 1].clone();
        for i in (0..debug_drawer.lines.len()).rev() {
            let line = &debug_drawer.lines[i];
            if line.color == first_line.color && line.weight == first_line.weight {
                path_builder.move_to(line.start);
                path_builder.line_to(line.end);
                debug_drawer.lines.swap_remove(i);
            }
        }
        let lines = path_builder.build();
        let mut geometry = GeometryBuilder::build_as(
            &PathBuilder::new().build(),
            DrawMode::Stroke(StrokeMode::new(
                first_line.color,
                first_line.weight * scalar,
            )),
            Transform::from_translation(Vec3::new(0., 0., 700.)),
        );
        geometry.path = lines;
        debug_drawer
            .test_entities
            .push(commands.spawn(geometry).id());
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
            transform::PhantomShape::Rectangle(Vec2::from_slice(&min), Vec2::from_slice(&max));

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
    q: Query<(&Transformable, &skin::Skin, Entity)>,
    skeleton: Res<skeleton::Skeleton>,
    mut debug_drawer: ResMut<DebugDrawer>,
    transform_state: Res<transform::State>,
    egui_state: Res<egui::State>,
) {
    if !debug_drawer.mesh_debug_enabled {
        return;
    };

    for (transformable, skin, entity) in q.iter() {
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

        let mut skin_mapping_index: Option<usize> = None;
        for i in 0..skeleton.skin_mappings.len() {
            if let Some(skin_entity) = skeleton.skin_mappings[i].skin {
                if skin_entity == entity {
                    skin_mapping_index = Some(i);
                }
            }
        }

        // draw VERTICES
        for i in 0..vertices.len() {
            // Determine Vertex display color
            let mut vertex_color = Color::rgb(0.0, 0.0, 0.0);
            let mut vertex_size: f32 = 5.0;
            if skin_mapping_index.is_some() && egui_state.adjust_vertex_weights_mode {
                if transform_state.selected_entities.len() > 0 {
                    // Currently selected bone
                    let bone_entity = *transform_state.selected_entities.iter().next().unwrap();
                    let v_mapping =
                        &skeleton.skin_mappings[skin_mapping_index.unwrap()].vertex_mappings[i];

                    // Weight of current bone for current vertex
                    for j in 0..v_mapping.bones.len() {
                        if v_mapping.bones[j] == bone_entity {
                            let weight = v_mapping.weights[j];
                            vertex_color = Color::rgb(weight, 1.0 - weight, 0.0);
                            vertex_size = 10.0;
                            break;
                        }
                    }
                }
            } else {
                vertex_color = color;
            };

            let vertex_size = if egui_state.adjust_vertex_weights_mode {
                vertex_size
            } else {
                5.0
            };
            debug_drawer.square(vertices[i].truncate(), vertex_size, vertex_color);
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
    cursor_pos: Res<CursorPos>,
    mut set: ParamSet<(
        Query<Entity, With<bone::Bone>>,
        Query<(&Transform, Option<&Parent>), With<bone::Bone>>,
        Query<&Transformable, With<bone::Bone>>,
    )>,
    egui_state: Res<egui::State>,
) {
    if !debug_drawer.bone_debug_enabled {
        return;
    };

    if egui_state.adjust_vertex_weights_mode {
        let mut angle = 0.0;
        while angle < (2.0 * std::f32::consts::PI) {
            let last_angle = angle;
            angle += f32::min(
                std::f32::consts::PI / 10.,
                std::f32::consts::PI / (12.0 * egui_state.brush_size),
            );
            let v_diff_last =
                Vec2::new(0.0, egui_state.brush_size).rotate(Vec2::from_angle(last_angle));
            let v_diff = Vec2::new(0.0, egui_state.brush_size).rotate(Vec2::from_angle(angle));
            debug_drawer.line(
                cursor_pos.0 + v_diff_last,
                cursor_pos.0 + v_diff,
                COLOR_WHITE,
            );
        }
    }

    let bone_entities: Vec<Entity> = set.p0().iter().collect();
    for entity in bone_entities {
        let opt_bone_gl_transform = bone::get_bone_gl_transform(entity, &set.p1());
        if let Some(gl_transform) = opt_bone_gl_transform {
            if gl_transform.translation.is_nan()
                || gl_transform.rotation.is_nan()
                || gl_transform.scale.is_nan()
            {
                dbg!(gl_transform);
                println!("transform is nan!");
                continue;
            }

            let z = 0.001;
            let color = if set.p2().get(entity).unwrap().is_selected {
                if set.p2().get(entity).unwrap().is_part_of_layer {
                    COLOR_SELECTED_ACTIVE
                } else {
                    COLOR_SELECTED
                }
            } else {
                if set.p2().get(entity).unwrap().is_part_of_layer {
                    COLOR_DEFAULT_ACTIVE
                } else {
                    COLOR_DEFAULT
                }
            };
            let mut points = vec![
                Vec3::new(0., 0., z),
                Vec3::new(-0.1, 0.1, z),
                Vec3::new(0., 1., z),
                Vec3::new(0.1, 0.1, z),
                Vec3::new(0., 0., z),
            ];
            for i in 0..points.len() {
                points[i].x *= gl_transform.scale.x;
                points[i].y *= gl_transform.scale.y;
            }
            for i in 0..points.len() {
                debug_drawer.line_thick(
                    (gl_transform.translation + Quat::mul_vec3(gl_transform.rotation, points[i]))
                        .truncate(),
                    (gl_transform.translation
                        + Quat::mul_vec3(gl_transform.rotation, points[(i + 1) % points.len()]))
                    .truncate(),
                    color,
                    5.,
                );
            }
            debug_drawer.square(gl_transform.translation.truncate(), 7., color);
        }
    }
}

pub fn enable_debug_lines(keys: Res<Input<KeyCode>>, mut debug_drawer: ResMut<DebugDrawer>) {
    if keys.just_pressed(KeyCode::B) {
        debug_drawer.bone_debug_enabled = !debug_drawer.bone_debug_enabled;
    }
    if keys.just_pressed(KeyCode::M) {
        debug_drawer.mesh_debug_enabled = !debug_drawer.mesh_debug_enabled;
    }
}

pub fn draw_ccd_target(
    debug_drawer: Res<DebugDrawer>,
    mut q: Query<(&Transformable, &mut Visibility, &mut Sprite), With<inverse_kinematics::Target>>,
) {
    for (transformable, mut visibility, mut sprite) in q.iter_mut() {
        if debug_drawer.bone_debug_enabled {
            visibility.is_visible = true;
        } else {
            visibility.is_visible = false;
            continue;
        }
        sprite.color = if transformable.is_selected {
            if transformable.is_part_of_layer {
                COLOR_SELECTED_ACTIVE
            } else {
                COLOR_SELECTED
            }
        } else {
            if transformable.is_part_of_layer {
                COLOR_DEFAULT_ACTIVE
            } else {
                COLOR_DEFAULT
            }
        };
    }
}

pub fn draw_select_box(
    mut debug_drawer: ResMut<DebugDrawer>,
    transform_state: Res<transform::State>,
    cursor_pos: Res<CursorPos>,
    clear_color: Res<ClearColor>,
    mut q: Query<(&mut Transform, &mut Visibility), With<misc::SelectBox>>,
) {
    let a = transform_state.cursor_anchor;
    let b = cursor_pos.0;
    if transform_state.drag_select {
        for (mut transform, mut visibility) in q.iter_mut() {
            transform.translation = ((a + b) / 2.).extend(800.);
            transform.scale = Vec3::new((a.x - b.x).abs(), (a.y - b.y).abs(), 1.);
            visibility.is_visible = true;
        }
        let color = bevy_image::ColorUtils::invert(&clear_color.0);
        debug_drawer.line_thick(a, Vec2::new(a.x, b.y), color, 2.0);
        debug_drawer.line_thick(a, Vec2::new(b.x, a.y), color, 2.0);
        debug_drawer.line_thick(b, Vec2::new(a.x, b.y), color, 2.0);
        debug_drawer.line_thick(b, Vec2::new(b.x, a.y), color, 2.0);
    } else {
        for (_, mut visibility) in q.iter_mut() {
            visibility.is_visible = false;
        }
    }
}
