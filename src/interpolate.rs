use std::fmt::Display;
use std::ops::*;
use serde::*;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, Copy)]
pub enum Function {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    EaseOutElastic,
    EaseInOutElastic,
    EaseInOutBack,
}
impl Function {
    /// Get a vector containing all interpolation functions
    pub fn all() -> impl ExactSizeIterator<Item = Function> {
        [
            Self::Linear,
            Self::EaseIn,
            Self::EaseOut,
            Self::EaseInOut,
            Self::EaseOutElastic,
            Self::EaseInOutElastic,
            Self::EaseInOutBack,
        ]
        .iter()
        .copied()
    }
}
impl ToString for Function {
    fn to_string(&self) -> String {
        match self {
            Function::Linear => String::from("linear"),
            Function::EaseIn => String::from("ease in"),
            Function::EaseOut => String::from("ease out"),
            Function::EaseInOut => String::from("ease in out"),
            Function::EaseOutElastic => String::from("ease out elastic"),
            Function::EaseInOutElastic => String::from("ease in out elastic"),
            Function::EaseInOutBack => String::from("ease in out back"),
        }
    }
}

pub fn lerp<T>(a: T, b: T, x: f32) -> T
where
    T: Add<T, Output = T>,
    T: Mul<f32, Output = T>,
{
    a * (1. - x) + b * x
}

// interpolation functions are taken from:
// https://www.febucci.com/2018/08/easing-functions/
// and
// https://easings.net/
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

pub fn ease_in_out_elastic(x: f32) -> f32 {
    let constant: f32 = (2. * std::f32::consts::PI) / 4.5;

    if x == 0. || x == 1. {
        x
    } else {
        if x < 0.5 {
            -(2f32.powf(20. * x - 10.) * ((20. * x - 11.125) * constant).sin()) / 2.
        } else {
            (2f32.powf(-20. * x + 10.) * ((20. * x - 11.125) * constant).sin()) / 2. + 1.
        }
    }
}

pub fn ease_in_out_back(x: f32) -> f32 {
    let c1 = 1.70158;
    let c2 = c1 * 1.525;

    if x < 0.5 {
        ((2. * x).powi(2) * ((c2 + 1.) * 2. * x - c2)) / 2.
    } else {
        ((2. * x - 2.).powi(2) * ((c2 + 1.) * (x * 2. - 2.) + c2) + 2.) / 2.
    }
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
