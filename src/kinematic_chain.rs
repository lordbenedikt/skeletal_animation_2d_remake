use crate::*;

#[cfg(test)]
#[path = "tests/kinematic_chain_tests.rs"]
mod kinematic_chain_tests;

/// Returns the global transform of element with given index in the kinematic chain.
///
/// The chain is ordered from leaf to root.
pub fn get_gl_transform(index: usize, chain: &Vec<Transform>) -> Transform {
    let mut gl_transform = chain[index];
    for i in (index + 1)..chain.len() {
        gl_transform = combined_transform(&chain[i], &gl_transform);
    }
    gl_transform
}

/// Get the tip of the link represented by the [`Transform`]
pub fn get_tip(transform: &Transform) -> Vec3 {
    transform.translation
        + transform
            .rotation
            .mul_vec3(Vec3::new(0.0, transform.scale.y, 0.0))
}

/// Get the tip of the kinematic chain
///
/// The chain is ordered from leaf to root.
pub fn get_tip_chain(chain: &Vec<Transform>) -> Vec3 {
    let gl_transform = get_gl_transform(0, chain);
    get_tip(&gl_transform)
}

pub fn get_chain_length(chain: &Vec<Transform>) -> f32 {
    let mut length = 0.0;
    let mut scale = Vec3::splat(1.0);

    for i in (0..chain.len()).rev() {
        // Don't add transform of first link
        if i != chain.len() - 1 {
            length += (chain[i].translation * scale).length();
        }

        scale *= chain[i].scale;

        // Add length of last link
        if i == 0 {
            length += scale.y;
        }
    }

    length
}
