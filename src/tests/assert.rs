use crate::*;

const EPSILON: f32 = 0.00001;

pub fn assert_scale_eq(a: Vec3, b: Vec3) {
    if (a - b).length() > EPSILON {
        panic!("Scales aren't equal: \n{}, \n{}", a, b);
    }
}

pub fn assert_translation_eq(a: Vec3, b: Vec3) {
    if (a - b).length() > EPSILON {
        panic!("Translations aren't equal: \n{}, \n{}", a, b);
    }
}

pub fn assert_quat_eq(a: &Quat, b: &Quat) {
    let a_negated = Quat::from_xyzw(-a.x, -a.y, -a.z, -a.w);
    let a_eq_b = (a.x - b.x).abs() < EPSILON
        && (a.y - b.y).abs() < EPSILON
        && (a.z - b.z).abs() < EPSILON
        && (a.w - b.w).abs() < EPSILON;
    let a_negated_eq_b = (a_negated.x - b.x).abs() < EPSILON
        && (a_negated.y - b.y).abs() < EPSILON
        && (a_negated.z - b.z).abs() < EPSILON
        && (a_negated.w - b.w).abs() < EPSILON;

    if !a_eq_b && !a_negated_eq_b {
        panic!("Quaternions aren't equal: \n{}, \n{}", a, b);
    }
}

pub fn assert_transform_eq(a: &Transform, b: &Transform) {
    assert_scale_eq(a.scale, b.scale);
    assert_translation_eq(a.translation, b.translation);
    assert_quat_eq(&a.rotation, &b.rotation);
}
