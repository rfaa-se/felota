use raylib::prelude::*;

pub fn rotate(point: Vector2, origin: Vector2, sin: f32, cos: f32) -> Vector2 {
    Vector2 {
        x: (cos * (point.x - origin.x)) - (sin * (point.y - origin.y)) + origin.x,
        y: (sin * (point.x - origin.x)) + (cos * (point.y - origin.y)) + origin.y,
    }
}

pub fn intersection(a: Vector2, b: Vector2, c: Vector2, d: Vector2) -> Option<Vector2> {
    fn area(a: Vector2, b: Vector2, c: Vector2) -> f32 {
        (a.x - c.x) * (b.y - c.y) - (a.y - c.y) * (b.x - c.x)
    }

    let a1 = area(a, b, c);
    let a2 = area(a, b, d);

    if a1 * a2 >= 0.0 {
        return None;
    }

    let a3 = area(c, d, a);
    let a4 = a3 + a2 - a1;

    if a3 * a4 >= 0.0 {
        return None;
    }

    let t = a3 / (a3 - a4);

    Some(a + (b - a) * t)
}
