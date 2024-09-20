use super::*;

use crate::math::*;

pub trait Shape: Renewable + Acceleratable + Rotatable {}

pub trait Vertexable {
    fn vertexes(&self, rotation: Vector2) -> Vec<Vector2>;
}

pub trait Boundable {
    fn bounds(&self) -> Rectangle;
}

pub trait Renewable {
    fn renew(&mut self);
}

pub trait Centroidable {
    fn centroid(&self) -> Vector2;
}

pub trait Rotatable {
    fn rotate(&mut self, amount: f32);
}

pub trait Lerpable<T> {
    fn lerp(&self, amount: f32) -> T;
}

pub trait Generationable {
    fn generation(&mut self);
}

pub trait Acceleratable {
    fn accelerate(&mut self, by: Vector2);
}

pub trait Cullable {
    fn cull(&self, viewport: Rectangle) -> bool;
}

impl Shape for Body<Triangle> {}
impl Shape for Body<Rectangle> {}
impl Shape for Body<Vector2> {}

impl Cullable for Rectangle {
    fn cull(&self, viewport: Rectangle) -> bool {
        !viewport.check_collision_recs(&self)
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
    fn lerp(&self, amount: f32) -> Triangle {
        Triangle {
            v1: self.old.shape.v1.lerp(self.new.shape.v1, amount),
            v2: self.old.shape.v2.lerp(self.new.shape.v2, amount),
            v3: self.old.shape.v3.lerp(self.new.shape.v3, amount),
        }
    }
}

impl Lerpable<Rectangle> for Generation<RotatedShape<Rectangle>> {
    fn lerp(&self, amount: f32) -> Rectangle {
        let old = &self.old.shape;
        let new = &self.new.shape;
        let xy = Vector2::new(old.x, old.y).lerp(Vector2::new(new.x, new.y), amount);
        let wh =
            Vector2::new(old.width, old.height).lerp(Vector2::new(new.width, new.height), amount);

        Rectangle {
            x: xy.x,
            y: xy.y,
            width: wh.x,
            height: wh.y,
        }
    }
}

impl Lerpable<Rectangle> for Generation<Rectangle> {
    fn lerp(&self, amount: f32) -> Rectangle {
        let xy =
            Vector2::new(self.old.x, self.old.y).lerp(Vector2::new(self.new.x, self.new.y), amount);
        let wh = Vector2::new(self.old.width, self.old.height)
            .lerp(Vector2::new(self.new.width, self.new.height), amount);

        Rectangle {
            x: xy.x,
            y: xy.y,
            width: wh.x,
            height: wh.y,
        }
    }
}

impl Lerpable<Vector2> for Generation<RotatedShape<Vector2>> {
    fn lerp(&self, amount: f32) -> Vector2 {
        self.old.shape.lerp(self.new.shape, amount)
    }
}

impl<T> Rotatable for Body<T> {
    fn rotate(&mut self, amount: f32) {
        let rot = &mut self.state.new.rotation;
        let rad = rot.y.atan2(rot.x) + amount;

        (rot.y, rot.x) = rad.sin_cos();

        if amount != 0.0 {
            self.polygon.dirty = true;
        }
    }
}

impl Acceleratable for Triangle {
    fn accelerate(&mut self, by: Vector2) {
        self.v1 += by;
        self.v2 += by;
        self.v3 += by;
    }
}

impl Acceleratable for Rectangle {
    fn accelerate(&mut self, by: Vector2) {
        self.x += by.x;
        self.y += by.y;
    }
}

impl Acceleratable for Vector2 {
    fn accelerate(&mut self, by: Vector2) {
        self.x += by.x;
        self.y += by.y;
    }
}

impl<T> Acceleratable for Body<T>
where
    T: Acceleratable,
{
    fn accelerate(&mut self, by: Vector2) {
        self.state.new.shape.accelerate(by);

        if by.x != 0.0 || by.y != 0.0 {
            self.polygon.dirty = true;
        }
    }
}

impl Vertexable for Triangle {
    fn vertexes(&self, rotation: Vector2) -> Vec<Vector2> {
        let ori = self.centroid();
        let (sin, cos) = rotation.y.atan2(rotation.x).sin_cos();

        vec![
            rotate(self.v1, ori, sin, cos),
            rotate(self.v2, ori, sin, cos),
            rotate(self.v3, ori, sin, cos),
        ]
    }
}

impl Vertexable for Rectangle {
    fn vertexes(&self, rotation: Vector2) -> Vec<Vector2> {
        let ori = self.centroid();
        let (sin, cos) = rotation.y.atan2(rotation.x).sin_cos();
        let x = self.x;
        let y = self.y;
        let w = self.width;
        let h = self.height;

        vec![
            rotate(Vector2::new(x, y), ori, sin, cos),
            rotate(Vector2::new(x + w, y), ori, sin, cos),
            rotate(Vector2::new(x + w, y + h), ori, sin, cos),
            rotate(Vector2::new(x, y + h), ori, sin, cos),
        ]
    }
}

impl Vertexable for Vector2 {
    fn vertexes(&self, _rotation: Vector2) -> Vec<Vector2> {
        vec![*self]
    }
}

impl Boundable for Vec<Vector2> {
    fn bounds(&self) -> Rectangle {
        let mut min_x = 0.0;
        let mut min_y = 0.0;
        let mut max_x = 0.0;
        let mut max_y = 0.0;

        if self.len() > 0 {
            min_x = self[0].x;
            max_x = min_x;
            min_y = self[0].y;
            max_y = min_y;

            for i in 1..self.len() {
                let vec = self[i];
                if vec.x < min_x {
                    min_x = vec.x;
                }

                if vec.y < min_y {
                    min_y = vec.y;
                }

                if vec.x > max_x {
                    max_x = vec.x;
                }

                if vec.y > max_y {
                    max_y = vec.y;
                }
            }
        }

        Rectangle {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    }
}

impl Boundable for Generation<Rectangle> {
    fn bounds(&self) -> Rectangle {
        let (min_x, max_x) = minmax(self.old.x, self.new.x);
        let (min_y, max_y) = minmax(self.old.y, self.new.y);
        let max_w = self.old.width.max(self.new.width);
        let max_h = self.old.height.max(self.new.height);

        fn minmax(one: f32, two: f32) -> (f32, f32) {
            if one < two {
                (one, two)
            } else {
                (two, one)
            }
        }

        Rectangle {
            x: min_x,
            y: min_y,
            width: max_x + max_w - min_x,
            height: max_y + max_h - min_y,
        }
    }
}

impl<T> Renewable for Body<T>
where
    T: Vertexable,
{
    fn renew(&mut self) {
        if !self.polygon.dirty {
            return;
        }

        self.polygon.vertexes.new = self.state.new.shape.vertexes(self.state.new.rotation);
        self.polygon.bounds_real.new = self.polygon.vertexes.new.bounds();
        self.polygon.bounds_meld.new = self.polygon.bounds_real.bounds();

        self.polygon.dirty = false;
    }
}

impl<T> Generationable for Generation<T>
where
    T: Copy,
{
    fn generation(&mut self) {
        self.old = self.new;
    }
}

impl<T> Generationable for Body<T>
where
    T: Copy,
{
    fn generation(&mut self) {
        self.state.generation();
        self.polygon.vertexes.old = self.polygon.vertexes.new.clone();
        self.polygon.bounds_real.generation();
        self.polygon.bounds_meld.generation();
    }
}
