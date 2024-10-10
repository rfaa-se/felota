use raylib::prelude::*;

use crate::{
    constants::{HUD_HEIGHT, RENDER_HEIGHT, RENDER_WIDTH},
    math::intersection,
};

pub fn generate_targeting_area(origin: Vector2, rotation: Vector2, angle: f32) -> Vec<Vector2> {
    let mut v = Vec::new();
    let radians = rotation.y.atan2(rotation.x);
    let big = 10000.0;
    let epsilon = 0.5;
    let viewport = Rectangle {
        x: origin.x - RENDER_WIDTH as f32 / 2.0,
        y: origin.y - RENDER_HEIGHT as f32 / 2.0 + HUD_HEIGHT as f32 / 2.0,
        width: RENDER_WIDTH as f32,
        height: RENDER_HEIGHT as f32 - HUD_HEIGHT as f32,
    };

    let (y, x) = (radians - angle).sin_cos();
    let end = origin + Vector2::new(x, y) * big;
    let left = find_intersection(origin, end, viewport);

    let (y, x) = (radians + angle).sin_cos();
    let end = origin + Vector2::new(x, y) * big;
    let right = find_intersection(origin, end, viewport);

    v.push(origin);
    v.push(left);
    v.push(right);

    let vx = viewport.x;
    let vy = viewport.y;
    let vw = viewport.width;
    let vh = viewport.height;
    let vxw = vx + vw;
    let vyh = vy + vh;

    let equal = |a: f32, b: f32| (a - b).abs() < epsilon;
    let between = |a, b, len| a > b && a < b + len;
    let closed = |a: Vector2, b: Vector2| {
        if equal(a.x, b.x) {
            // cannot be closed if not by the edge
            if equal(a.x, vx) || equal(a.x, vxw) {
                return true;
            }
        }

        if equal(a.y, b.y) {
            // cannot be closed if not by the edge
            if equal(a.y, vy) || equal(a.y, vyh) {
                return true;
            }
        }

        false
    };

    // add new vertexes until we have a closed polygon,
    // start by going with the left vertex and keep adding until we reach the right vertex,
    // vertexes will be placed along the viewport
    let mut open = !closed(left, right);

    while open {
        let prev = v[v.len() - 2];

        let new = if equal(prev.x, vx) && between(prev.y, vy, vh) {
            Vector2::new(vx, vy)
        } else if equal(prev.x, vxw) && between(prev.y, vy, vh) {
            Vector2::new(vxw, vyh)
        } else if equal(prev.y, vy) && between(prev.x, vx, vw) {
            Vector2::new(vxw, vy)
        } else if equal(prev.y, vyh) && between(prev.x, vx, vw) {
            Vector2::new(vx, vyh)
        } else {
            if prev.x == vx && prev.y == vy {
                Vector2::new(vxw, vy)
            } else if prev.x == vxw && prev.y == vy {
                Vector2::new(vxw, vyh)
            } else if prev.x == vx && prev.y == vyh {
                Vector2::new(vx, vy)
            } else {
                Vector2::new(vx, vyh)
            }
        };

        v.insert(v.len() - 1, new);

        open = !closed(new, right);
    }

    v
}

fn find_intersection(start: Vector2, end: Vector2, viewport: Rectangle) -> Vector2 {
    if let Some(x) = intersection(
        start,
        end,
        Vector2::new(viewport.x, viewport.y),
        Vector2::new(viewport.x + viewport.width, viewport.y),
    ) {
        return x;
    }

    if let Some(x) = intersection(
        start,
        end,
        Vector2::new(viewport.x + viewport.width, viewport.y),
        Vector2::new(viewport.x + viewport.width, viewport.y + viewport.height),
    ) {
        return x;
    }

    if let Some(x) = intersection(
        start,
        end,
        Vector2::new(viewport.x + viewport.width, viewport.y + viewport.height),
        Vector2::new(viewport.x, viewport.y + viewport.height),
    ) {
        return x;
    }

    if let Some(x) = intersection(
        start,
        end,
        Vector2::new(viewport.x, viewport.y + viewport.height),
        Vector2::new(viewport.x, viewport.y),
    ) {
        return x;
    }

    panic!("wtf big viewport or crazy start/end");
}
