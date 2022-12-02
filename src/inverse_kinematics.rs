use std::f32::consts::PI;

use crate::{animation::Animatable, *};
use bone::Bone;
use serde::{Serialize, Deserialize};

extern crate nalgebra as na;

#[derive(PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum IKMethod {
    CCD,
    Jacobian,
}
impl IKMethod {
    /// Get a vector containing all interpolation functions
    pub fn all() -> impl ExactSizeIterator<Item = IKMethod> {
        [Self::CCD, Self::Jacobian].iter().copied()
    }
}
impl ToString for IKMethod {
    fn to_string(&self) -> String {
        match self {
            Self::CCD => String::from("Cyclic Coordinate Descent"),
            Self::Jacobian => String::from("Jacobian Pseudo Inverse"),
        }
    }
}

#[derive(Component, Clone)]
pub struct Target {
    pub ik_method: IKMethod,
    pub bone: Entity,
    pub depth: u8,
}

pub fn system_set() -> SystemSet {
    SystemSet::new()
        .with_system(add_target)
        .with_system(reach_for_target)
}

pub fn add_target(
    mut commands: Commands,
    mut q: Query<(Entity, &mut Transformable), With<Bone>>,
    cursor_pos: Res<CursorPos>,
    keys: Res<Input<KeyCode>>,
    mouse: Res<Input<MouseButton>>,
    mut transform_state: ResMut<transform::State>,
    egui_state: Res<egui::State>,
    asset_server: Res<AssetServer>,
) {
    // Add IK Target only if Alt + Left Mouse was pressed
    if !keys.pressed(KeyCode::LAlt) || !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let mut opt_bone_entity: Option<Entity> = None;
    for (bone_entity, transformable) in q.iter() {
        if transformable.is_selected {
            opt_bone_entity = Some(bone_entity);
            break;
        }
    }
    if let Some(bone_entity) = opt_bone_entity {
        transform_state.selected_entities.clear();
        transform_state.selected_entities.insert(
            commands
                .spawn_bundle(SpriteBundle {
                    transform: Transform::default().with_translation(cursor_pos.0.extend(500.)),
                    sprite: Sprite {
                        color: COLOR_DEFAULT,
                        custom_size: Some(Vec2::new(0.4, 0.4)),
                        anchor: bevy::sprite::Anchor::Center,
                        ..Default::default()
                    },
                    texture: asset_server.load("img/ccd_target.png"),
                    ..Default::default()
                })
                .insert(Target {
                    ik_method: egui_state.ik_method,
                    bone: bone_entity,
                    depth: egui_state.ik_depth,
                })
                .insert(Transformable {
                    collision_shape: transform::PhantomShape::Point,
                    ..Default::default()
                })
                .insert(Animatable)
                .id(),
        );
    } else {
        return;
    }
    for (_, mut transformable) in q.iter_mut() {
        transformable.is_selected = false;
    }
    transform_state.action = Action::Done;
}

/// Returns a vector of rotation quaternions, at which `chain` reaches target.
///
/// The algorithm ends as soon as `eps` greater or equal to the distance of the end effector to `target`,
/// or `max_it` iterations have been executed.
fn get_target_rotations_ccd(
    mut chain: Vec<Transform>,
    constraints: Vec<Option<bone::AngleConstraint>>,
    target: Vec2,
    eps: f32,
    max_it: usize,
) -> Vec<Quat> {
    // Get tip of chain
    let mut end_effector_pos = kinematic_chain::get_tip_chain(&chain).truncate();

    // Perform CCD
    for _ in 0..max_it {
        for i in 0..chain.len() {
            // Rotate current bone so that current_pos, end_effector_pos and target are on one line
            let current_pos = kinematic_chain::get_gl_transform(i, &chain)
                .translation
                .truncate();

            let delta_rot = Quat::from_rotation_arc_2d(
                (end_effector_pos - current_pos).normalize(),
                (target - current_pos).normalize(),
            );
            chain[i].rotation = (chain[i].rotation * delta_rot).normalize();

            if let Some(c) = &constraints[i] {
                if c.start != c.end {
                    let rot = (chain[i].rotation.to_euler(EulerRot::XYZ).2 + (4.*PI)) % (2.*PI);
                    if !((rot >= c.start && rot <= c.end)
                        || (rot >= c.start - 2. * PI && rot <= c.end - 2. * PI))
                    {
                        let start_dist = (rot - (c.start % (2. * PI))).abs();
                        let end_dist = (rot - (c.end % (2. * PI))).abs();
                        let fixed_angle = if start_dist <= end_dist {
                            c.start
                        } else {
                            c.end
                        };
                        chain[i].rotation = Quat::from_rotation_z(fixed_angle);
                    }
                }
            }

            end_effector_pos = kinematic_chain::get_tip_chain(&chain).truncate();
        }
        if end_effector_pos.distance(target) <= eps {
            break;
        }
    }
    chain.iter().map(|transform| transform.rotation).collect()
}

pub fn get_target_rotations_jacobian(
    mut chain: Vec<Transform>,
    target: Vec2,
    eps: f32,
    step: f32,
    max_it: usize,
) -> Vec<Quat> {
    let mut v = Vec3::new(0.0, 0.0, 0.0);

    // let get_pseudo_inverse = ||

    let get_jacobian_pseudo_inverse = |v: Vec3, chain: &Vec<Transform>| -> na::DMatrix<f32> {
        let rot_axis = Vec3::new(0.0, 0.0, 1.0);

        let mut res = na::DMatrix::<f32>::zeros(3, chain.len());

        for i in 0..chain.len() {
            let end_effector_pos = kinematic_chain::get_tip_chain(&chain);
            let joint_pos = kinematic_chain::get_gl_transform(i, &chain).translation;
            let cross_product = animation::cross_product(rot_axis, end_effector_pos - joint_pos);
            res[(0, i)] = cross_product.x;
            res[(1, i)] = cross_product.y;
            res[(2, i)] = cross_product.z;
        }

        res.pseudo_inverse(0.001).unwrap()
        // res.transpose()
    };

    let get_delta_angles = |v: Vec3, chain: &Vec<Transform>| -> Vec<f32> {
        let mut delta_rotations = vec![];
        let jacobian_pseudo_inverse = get_jacobian_pseudo_inverse(v, chain);

        for jtp_row in jacobian_pseudo_inverse.row_iter() {
            let row = Vec3::new(jtp_row[0], jtp_row[1], jtp_row[2]);
            delta_rotations.push(animation::dot_product(row, v));
        }
        return delta_rotations;
    };

    let chain_length = kinematic_chain::get_chain_length(&chain);
    let mut target_arranged = target;
    for _ in 0..max_it {
        let end_effector_pos = kinematic_chain::get_tip_chain(&chain);

        let root_to_target = target.extend(0.) - chain.last().unwrap().translation;
        let scalar = chain_length / root_to_target.length();
        // println!("chain_len: {}",chain_length);

        if scalar < 1.0 {
            // println!("scalar: {}",scalar);
            let v_diff = root_to_target * (scalar - 1.002);
            target_arranged = target + v_diff.truncate();
        }
        // let scalar =  chain_length / v.length();
        // println!("v_len: {}",end_effector_pos.length());
        // println!("chain_len: {}",chain_length);
        // println!("scalar: {}", scalar);
        // if scalar < 1.0 {
        //     println!("scaling..");
        //     v *= scalar;
        // }

        v = target_arranged.extend(0.0) - end_effector_pos;
        if v.length() <= eps {
            break;
        }

        let delta_rotations = get_delta_angles(v, &chain);

        for i in 0..chain.len() {
            chain[i].rotation *= Quat::from_rotation_z(delta_rotations[i] * step);
        }
    }

    chain.iter().map(|transform| transform.rotation).collect()
}

pub fn reach_for_target(
    mut commands: Commands,
    mut q_bones: Query<(&mut Transform, Option<&Parent>, &mut Bone)>,
    egui_state: Res<egui::State>,
    q_targets: Query<(Entity, &Transform, &Target), Without<Bone>>,
) {
    // Reset bone.is_ik_maneuvered
    for (_, _, mut bone) in q_bones.iter_mut() {
        bone.is_ik_maneuvered = false;
    }

    // For each TARGET
    for (entity, target_transform, target) in q_targets.iter() {
        // Construct chain from bones query
        let mut chain_transforms = vec![];
        let mut chain_entities = vec![];
        let mut chain_constraints = vec![];
        let mut next_bone = target.bone;
        let mut chain_has_parent_bone = true;
        for _ in 0..target.depth {
            if let Ok((_, _, mut bone)) = q_bones.get_mut(next_bone) {
                // Mark bone as ik_maneuvered
                bone.is_ik_maneuvered = true;
            } else {
                // If bone was removed, despawn target
                commands.entity(entity).despawn();
                return;
            }

            let (transform, opt_parent, bone) = q_bones.get(next_bone).unwrap();

            // Add bone to chain and continue with parent, if present
            chain_transforms.push(transform.clone());
            chain_entities.push(next_bone);
            chain_constraints.push(bone.ik_angle_constraint.clone());
            next_bone = if let Some(parent) = opt_parent {
                parent.get()
            } else {
                chain_has_parent_bone = false;
                break;
            }
        }

        // Get root bone's global transform
        let mut root_gl_transform = Transform::default();
        loop {
            // If chain has no parent, break
            if !chain_has_parent_bone {
                break;
            }

            // If bone was removed, despawn target
            if q_bones.get(next_bone).is_err() {
                commands.entity(entity).despawn();
                return;
            }

            let (transform, opt_parent, _) = q_bones.get(next_bone).unwrap();

            root_gl_transform = combined_transform(transform, &root_gl_transform);
            next_bone = if let Some(parent) = opt_parent {
                parent.get()
            } else {
                break;
            }
        }

        // Get IK target relative to the root of the kinematic chain
        let target_pos = get_relative_transform(&root_gl_transform, &target_transform)
            .translation
            .truncate();

        let target_rotations = match target.ik_method {
            IKMethod::CCD => {
                get_target_rotations_ccd(chain_transforms, chain_constraints, target_pos, 0.01, egui_state.ik_max_iterations)
            }
            IKMethod::Jacobian => {
                get_target_rotations_jacobian(chain_transforms, target_pos, 0.01, 1.0, egui_state.ik_max_iterations)
            }
        };

        for i in 0..chain_entities.len() {
            q_bones.get_mut(chain_entities[i]).unwrap().0.rotation = target_rotations[i];
        }
    }
}

// fn get_delta_orientation_jacobian(v: Vec3) -> Vec<f32> {
//     let jacobian_transpose = get_jacobian_transpose();
//     let delta_orientation = jacobian_transpose * v;
//     delta_orientation
// }

// fn get_jacobian_transpose() -> bevy::prelude::Mat2 {
//     bevy::prelude::Mat2::IDENTITY;
// }

trait Vec2Angles {
    fn get_angle(self) -> f32;
    fn rotate_by(self, angle: f32) -> Self;
}

impl Vec2Angles for Vec2 {
    fn get_angle(self) -> f32 {
        let angle = self.angle_between(Vec2::Y);
        angle
    }
    fn rotate_by(self, angle: f32) -> Self {
        Vec2::from_angle(angle).rotate(self)
        // let x = self.x * angle.cos() + self.y * (-angle.sin());
        // let y = self.x * angle.sin() + self.y * angle.cos();
        // Vec2::new(x, y)
    }
}

#[derive(Clone)]
struct MatMN {
    m: usize,
    n: usize,
    data: Vec<Vec<f32>>,
}
impl MatMN {
    fn new(data: Vec<Vec<f32>>) -> Self {
        Self {
            m: data.len(),
            n: data[0].len(),
            data,
        }
    }
    fn mul(&self, other: &Self) -> Self {
        if self.n != other.m {
            panic!(
                "Number of first matrix's columns doesn't equal number of second matrix's rows!"
            );
        }
        let m = self.m;
        let n = other.n;
        let mut data: Vec<Vec<f32>> = vec![];

        for row in 0..m {
            data.push(vec![]);
            for col in 0..n {
                let mut value = 0.0;
                for x in 0..self.n {
                    value += self.data[row][x] * other.data[x][col];
                }
                data.last_mut().unwrap().push(value);
            }
        }

        Self::new(data)
    }
    fn inv(&self) -> Self {
        self.clone()
    }
    fn cofactor(&self, row: usize, col: usize) -> f32 {
        if self.m != self.n {
            panic!("Can't get cofactor of a non square matrix!");
        }
        if row >= self.m || col >= self.n {
            panic!("Element is out of matrix bounds!");
        }
        0.0
        // if self.m == 1 {
        //     return 1.0;
        // }
        // if self
    }
}

#[test]
fn mat_mn_mul_test() {
    let m1 = MatMN::new(vec![
        vec![1., 2., 3.],
        vec![2., 3., 4.],
        vec![3., 4., 5.],
        vec![1., 0., 1.],
    ]);
    let m2 = MatMN::new(vec![
        vec![2., 2., 2., 2.],
        vec![3., 0., 3., 0.],
        vec![5., 5., 5., 3.],
    ]);
    // println!("{}:{}", m1.m, m2.m);
    // dbg!(m1.mul(&m2).data);
}
