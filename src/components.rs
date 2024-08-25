use std::f32::consts::FRAC_PI_4;

use raylib::prelude::*;

pub mod traits;

pub use self::traits::*;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub v1: Vector2,
    pub v2: Vector2,
    pub v3: Vector2,
}

pub struct Body<T> {
    pub state: Generation<RotatedShape<T>>,
    pub color: Color,
    pub polygon: Polygon,
}

#[derive(Clone, Copy)]
pub struct RotatedShape<T> {
    pub shape: T,
    pub rotation: Vector2,
}

#[derive(Clone, Copy)]
pub struct Generation<T> {
    pub old: T,
    pub new: T,
}

pub struct Motion {
    pub velocity: Vector2,
    pub acceleration: f32,
    pub speed_max: f32,
    pub rotation_speed: f32,
    pub rotation_acceleration: f32,
    pub rotation_speed_max: f32,
}

pub struct Polygon {
    pub dirty: bool,
    pub vertexes: Generation<Vec<Vector2>>,
    pub bounds_real: Generation<Rectangle>,
    pub bounds_meld: Generation<Rectangle>,
}

// TODO: move me?
pub struct Direction;

#[allow(dead_code)]
impl Direction {
    pub const NORTH: Vector2 = Vector2 { x: 0.0, y: -1.0 };
    pub const SOUTH: Vector2 = Vector2 { x: 0.0, y: 1.0 };
    pub const EAST: Vector2 = Vector2 { x: 1.0, y: 0.0 };
    pub const WEST: Vector2 = Vector2 { x: -1.0, y: 0.0 };
    pub const NORTHWEST: Vector2 = Vector2 {
        x: -FRAC_PI_4,
        y: -FRAC_PI_4,
    };
    pub const NORTHEAST: Vector2 = Vector2 {
        x: FRAC_PI_4,
        y: -FRAC_PI_4,
    };
    pub const SOUTHEAST: Vector2 = Vector2 {
        x: FRAC_PI_4,
        y: FRAC_PI_4,
    };
    pub const SOUTHWEST: Vector2 = Vector2 {
        x: -FRAC_PI_4,
        y: FRAC_PI_4,
    };
}
