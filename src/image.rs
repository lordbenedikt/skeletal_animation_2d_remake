use crate::*;
use std::cmp;

pub trait Pixels {
    fn get_pixel(&self, x: u32, y: u32) -> &[u8];
    fn get_pixel_alpha(&self, x: u32, y: u32) -> u8;
    fn is_close_to_visible(&self, x: i32, y: i32, max_dist: f32) -> bool;
}
impl Pixels for Image {
    fn get_pixel(&self, x: u32, y: u32) -> &[u8] {
        let (w, _) = (self.size().x as u32, self.size().y as u32);
        let from = (x + y * w) as usize * 4;
        let to = (from + 4) as usize;
        &self.data[from..to]
    }
    fn get_pixel_alpha(&self, x: u32, y: u32) -> u8 {
        let (w, _) = (self.size().x as u32, self.size().y as u32);
        let from = (x + y * w) as usize * 4;
        let alpha = (from + 3) as usize;
        self.data[alpha]
    }
    fn is_close_to_visible(&self, x: i32, y: i32, max_dist: f32) -> bool {
        let max_dist_ceil = f32::ceil(max_dist) as i32;
        let (w, h) = (self.size().x as i32, self.size().y as i32);
        let x_min = cmp::max(0, x - max_dist_ceil);
        let x_max = cmp::min(w, x + max_dist_ceil);
        for _x in x_min..x_max {
            let y_min = cmp::max(0, y - max_dist_ceil);
            let y_max = cmp::min(h, y + max_dist_ceil);
            for _y in y_min..y_max {
                let square_distance = (x - _x).pow(2) + (y - _y).pow(2);
                let distance = (square_distance as f32).sqrt();
                // if distance is smaller same max_dist
                if distance <= max_dist {
                    // if pixel is not transparent
                    if self.get_pixel(_x as u32, _y as u32)[3] > 10 {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub trait ColorUtils {
    fn invert(&self) -> Self;
}
impl ColorUtils for Color {
    fn invert(&self) -> Self {
        let col = self.as_rgba();
        Color::rgba(1. - col.r(), 1. - col.g(), 1. - col.b(), 0.2)
    }
}
