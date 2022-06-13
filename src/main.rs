mod cursor;
mod setup;

use bevy::prelude::*;
use bevy_prototype_debug_lines::*;

const COLOR_SELECTED: Color = Color::rgb(1., 1., 1.);
const COLOR_DEFAULT: Color = Color::rgb(1., 0.6, 0.);

// COMPONENTs
#[derive(Component)]
struct Bone;

// RESOURCES
struct Bones(Vec<Entity>);
pub struct CursorPos(Vec2);

fn main() {
    App::new()
        // RESOURCES
        .insert_resource(WindowDescriptor {
            title: "Skeletal Animation".to_string(),
            width: 800.,
            height: 600.,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .insert_resource(Bones(Vec::new()))
        .insert_resource(CursorPos(Vec2::new(0., 0.)))
        // PLUGINS
        .add_plugins(DefaultPlugins)
        .add_plugin(DebugLinesPlugin::default())
        // STARTUP SYSTEMS
        .add_startup_system(setup::setup)
        // SYSTEMS
        .add_system(cursor::get_position.label("input_handling"))
        .add_system(add_bone.after("input_handling"))
        .add_system(remove_bone.before(add_bone))
        .add_system(draw_debug_lines)
        // RUN
        .run();
}

fn remove_bone(
    mut commands: Commands,
    mut bones: ResMut<Bones>,
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
    mut query: Query<(&Bone, Entity)>,
) {
    // // Add bone if left mouse was pressed
    // if !mouse.just_pressed(MouseButton::Left) {
    //     return;
    // }

    // for (bone, entity) in query.iter_mut() {
    //     commands.entity(entity).despawn_recursive();
    //     break;
    // }
}

fn draw_debug_lines(mut lines: ResMut<DebugLines>, bone_transforms: Query<&Transform, With<Bone>>) {
    for transform in bone_transforms.iter() {
        let z = 100.;
        let points = vec![
            Vec3::new(0., 0., z),
            Vec3::new(-0.1, 0.1, z),
            Vec3::new(0., 1., z),
            Vec3::new(0.1, 0.1, z),
            Vec3::new(0., 0., z),
        ];
        for i in 0..points.len() {
            lines.line(
                Quat::mul_vec3(Quat::from_rotation_z(transform.rotation.z), points[i]),
                Quat::mul_vec3(
                    Quat::from_rotation_z(transform.rotation.z),
                    points[(i + 1) % points.len()],
                ),
                0.,
            );
        }
    }
}

fn add_bone(
    mut commands: Commands,
    mut bones: ResMut<Bones>,
    mouse: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPos>,
) {
    // Add bone if left mouse was pressed
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    // Spawn sprite bundle and add id to Bones resource
    let entity = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.4, 0.4, 0.4),
                ..Default::default()
            },
            transform: Transform {
                translation: Vec3::new(cursor_pos.0.x, cursor_pos.0.y, 0.),
                scale: Vec3::new(1., 1., 0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Bone)
        // Add debug lines as child entity
        // .with_children(|parent| {
        //     parent.spawn_bundle(GeometryBuilder::build_as(
        //         &shapes::Polygon {
        //             points: vec![
        //                 Vec2::new(0., 0.),
        //                 Vec2::new(-0.1, 0.1),
        //                 Vec2::new(0., 1.),
        //                 Vec2::new(0.1, 0.1),
        //                 Vec2::new(0., 0.),
        //             ],
        //             closed: true,
        //         },
        //         DrawMode::Stroke {
        //             0: StrokeMode::new(COLOR_DEFAULT, 2. / 100.),
        //         },
        //         Transform {
        //             translation: Vec3::new(0., 0., 10.),
        //             scale: Vec3::new(1., 1., 1.),
        //             ..Default::default()
        //         },
        //     ));
        // })
        .id();
    bones.0.push(entity);
}
