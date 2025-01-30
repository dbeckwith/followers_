use std::ops::{Add, AddAssign, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vec2 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, scale: f32) -> Self::Output {
        Self {
            x: self.x * scale,
            y: self.y * scale,
        }
    }
}

impl Vec2 {
    pub fn dot(self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y
    }

    pub fn length_squared(self) -> f32 {
        self.dot(self)
    }

    pub fn distance_squared(self, other: Self) -> f32 {
        (self - other).length_squared()
    }

    pub fn clamp_length_max(self, max_length: f32) -> Self {
        let max_length_sq = max_length * max_length;
        let length_sq = self.x * self.x + self.y * self.y;
        if length_sq > max_length_sq {
            self * (max_length / length_sq.sqrt())
        } else {
            self
        }
    }
}

pub fn lerp(
    x: f32,
    old_min: f32,
    old_max: f32,
    new_min: f32,
    new_max: f32,
) -> f32 {
    (x - old_min) / (old_max - old_min) * (new_max - new_min) + new_min
}
