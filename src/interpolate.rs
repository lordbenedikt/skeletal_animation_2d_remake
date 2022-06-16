use std::ops::*;

pub fn lerp<T>(a: T, b: T, x: f32) -> T
where
    T: Add<T, Output = T>,
    T: Mul<f32, Output = T>,
{
    a * (1. - x) + b * x
}

pub fn ease_out_elastic(x: f32) -> f32 {
    if x == 0. {
        return 0.;
    }
    if x == 1. {
        return 1.;
    }
    let constant: f32 = (2. * std::f32::consts::PI) / 3.;
    (2 as f32).powf(-10. * x) * f32::sin((x * 10. - 0.75) * constant) + 1.
}

pub fn ease_in_out(x: f32) -> f32 {
    lerp(ease_in(x), ease_out(x), x)
}

pub fn ease_in(x: f32) -> f32 {
    x * x
}

pub fn ease_out(x: f32) -> f32 {
    let x_flipped = flip(x);
    flip(x_flipped * x_flipped)
}

pub fn flip(x: f32) -> f32 {
    1. - x
}