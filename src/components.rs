use std::f32::consts::FRAC_PI_4;

use raylib::prelude::*;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub v1: Vector2,
    pub v2: Vector2,
    pub v3: Vector2,
}

pub struct Body<T> {
    pub generation: Generation<RotatedShape<T>>,
    pub color: Color,
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

// TODO: move the traits and impls to somewhere else?

pub trait Centroidable {
    fn centroid(&self) -> Vector2;
}

pub trait Rotatable<T> {
    fn rotated(&self, angle: Vector2) -> T;
}

pub trait Lerpable<T> {
    fn lerp(&self, amount: f32) -> T;
}

pub trait Renewable {
    fn renew(&mut self);
}

pub trait Acceleratable {
    fn accelerate(&mut self, acceleration: Vector2);
}

pub trait Cullable {
    fn should_cull(&self, viewport: Rectangle) -> bool;
}

impl Cullable for Body<Rectangle> {
    fn should_cull(&self, viewport: Rectangle) -> bool {
        // this does not take rotation into consideration,
        // instead we create a square big enough to contain every rotation
        let s = self.generation.old.shape;
        let max = s.width.max(s.height);
        let max2 = max * 2.0;

        !viewport.check_collision_recs(&Rectangle {
            x: s.x - max,
            y: s.y - max,
            width: max2,
            height: max2,
        })
    }
}

impl Cullable for Body<Triangle> {
    fn should_cull(&self, viewport: Rectangle) -> bool {
        // this does not take rotation into consideration,
        // instead we create a square big enough to contain every rotation
        let s = self.generation.old.shape;
        let c = s.centroid();
        let x = (s.v1.x - s.v2.x).abs().max((s.v2.x - s.v3.x).abs());
        let y = (s.v1.y - s.v2.y).abs().max((s.v2.y - s.v3.y).abs());
        let max = x.max(y);
        let max2 = max * 2.0;

        !viewport.check_collision_recs(&Rectangle {
            x: c.x - max,
            y: c.y - max,
            width: max2,
            height: max2,
        })
    }
}

impl Cullable for Body<Vector2> {
    fn should_cull(&self, viewport: Rectangle) -> bool {
        !viewport.check_collision_point_rec(self.generation.old.shape)
    }
}

impl Centroidable for Triangle {
    fn centroid(&self) -> Vector2 {
        Vector2 {
            x: (self.v1.x + self.v2.x + self.v3.x) / 3.0,
            y: (self.v1.y + self.v2.y + self.v3.y) / 3.0,
        }
    }
}

impl Centroidable for Rectangle {
    fn centroid(&self) -> Vector2 {
        Vector2 {
            x: (self.x + self.width / 2.0),
            y: (self.y + self.height / 2.0),
        }
    }
}

impl Lerpable<Triangle> for Generation<RotatedShape<Triangle>> {
    fn lerp(&self, delta: f32) -> Triangle {
        Triangle {
            v1: self.old.shape.v1.lerp(self.new.shape.v1, delta),
            v2: self.old.shape.v2.lerp(self.new.shape.v2, delta),
            v3: self.old.shape.v3.lerp(self.new.shape.v3, delta),
        }
    }
}

impl Lerpable<Rectangle> for Generation<RotatedShape<Rectangle>> {
    fn lerp(&self, amount: f32) -> Rectangle {
        let v = Vector2::new(self.old.shape.x, self.old.shape.y)
            .lerp(Vector2::new(self.new.shape.x, self.new.shape.y), amount);
        // TODO: lerp width and height as well?
        Rectangle {
            x: v.x,
            y: v.y,
            width: self.new.shape.width,
            height: self.new.shape.height,
        }
    }
}

impl Lerpable<Vector2> for Generation<RotatedShape<Vector2>> {
    fn lerp(&self, amount: f32) -> Vector2 {
        self.old.shape.lerp(self.new.shape, amount)
    }
}

impl Rotatable<Triangle> for Triangle {
    fn rotated(&self, angle: Vector2) -> Triangle {
        let radians = angle.y.atan2(angle.x);
        let origin = self.centroid();
        let (sin, cos) = radians.sin_cos();

        Triangle {
            v1: rotate(self.v1, origin, sin, cos),
            v2: rotate(self.v2, origin, sin, cos),
            v3: rotate(self.v3, origin, sin, cos),
        }
    }
}

impl<T> Renewable for Generation<T>
where
    T: Copy,
{
    fn renew(&mut self) {
        self.old = self.new;
    }
}

impl Acceleratable for Triangle {
    fn accelerate(&mut self, acceleration: Vector2) {
        self.v1 += acceleration;
        self.v2 += acceleration;
        self.v3 += acceleration;
    }
}

impl Acceleratable for Rectangle {
    fn accelerate(&mut self, acceleration: Vector2) {
        self.x += acceleration.x;
        self.y += acceleration.y;
    }
}

impl Acceleratable for Vector2 {
    fn accelerate(&mut self, acceleration: Vector2) {
        self.x += acceleration.x;
        self.y += acceleration.y;
    }
}

pub fn rotate(point: Vector2, origin: Vector2, sin: f32, cos: f32) -> Vector2 {
    Vector2 {
        x: (cos * (point.x - origin.x)) - (sin * (point.y - origin.y)) + origin.x,
        y: (sin * (point.x - origin.x)) + (cos * (point.y - origin.y)) + origin.y,
    }
}

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
