use std::ops::Add;

use raylib::prelude::*;

use crate::{
    components::{Centroidable, Cullable, Lerpable, Triangle},
    constants::{
        COSMOS_HEIGHT, COSMOS_WIDTH, HUD_HEIGHT, RENDER_HEIGHT, RENDER_WIDTH, STARFIELD_HEIGHT,
        STARFIELD_WIDTH,
    },
    entities::Entities,
    math::*,
};

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn draw(
        &self,
        r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
        entities: &Entities,
        viewport: Rectangle,
        debug: bool,
        delta: f32,
    ) {
        if debug {
            r.draw_rectangle_lines_ex(viewport, 1.0, Color::RED);
        }

        r.draw_rectangle_lines(0, 0, COSMOS_WIDTH, COSMOS_HEIGHT, Color::RED);

        draw_stars(r, entities, viewport, delta);
        draw_exhausts(r, entities, viewport, delta);
        draw_triships(r, entities, viewport, debug, delta);
        draw_explosions(r, entities, viewport, delta);
        draw_projectiles(r, entities, viewport, delta);
        draw_torpedoes(r, entities, viewport, delta);
    }
}

fn draw_stars(
    r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
    entities: &Entities,
    viewport: Rectangle,
    delta: f32,
) {
    for star in &entities.stars {
        let gen = &star.entity.body.state;
        let mut ent = gen.lerp(delta);

        let min_x = if viewport.x < 0.0 {
            0.0
        } else {
            if viewport.x > COSMOS_WIDTH as f32 {
                COSMOS_WIDTH as f32
            } else {
                viewport.x
            }
        };

        let max_x = if viewport.x + viewport.width > COSMOS_WIDTH as f32 {
            COSMOS_WIDTH as f32
        } else {
            viewport.x + viewport.width
        };

        let min_y = if viewport.y < 0.0 {
            0.0
        } else {
            if viewport.y > COSMOS_HEIGHT as f32 {
                COSMOS_HEIGHT as f32
            } else {
                viewport.y
            }
        };

        let max_y = if viewport.y + viewport.height > COSMOS_HEIGHT as f32 {
            COSMOS_HEIGHT as f32
        } else {
            viewport.y + viewport.height
        };

        while ent.x < min_x {
            ent.x += STARFIELD_WIDTH as f32;
        }

        while ent.y < min_y {
            ent.y += STARFIELD_HEIGHT as f32;
        }

        if ent.x > max_x {
            continue;
        }

        if ent.y > max_y {
            continue;
        }

        let x = ent.x;

        loop {
            loop {
                r.draw_pixel_v(ent, star.entity.body.color);

                ent.x += STARFIELD_WIDTH as f32;

                if ent.x > max_x {
                    break;
                }
            }

            ent.y += STARFIELD_HEIGHT as f32;
            ent.x = x;

            if ent.y > max_y {
                break;
            }
        }
    }
}

fn draw_projectiles(
    r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
    entities: &Entities,
    viewport: Rectangle,
    delta: f32,
) {
    for projectile in &entities.projectiles {
        let bounds = projectile.entity.body.polygon.bounds_real.lerp(delta);

        if bounds.cull(viewport) {
            continue;
        }

        let gen = projectile.entity.body.state;
        let rot = gen.old.rotation.lerp(gen.new.rotation, delta);
        let rad = rot.y.atan2(rot.x);
        let deg = rad.to_degrees();
        let ent = gen.lerp(delta);

        // for some reason we need to add half the width and height to rotated rectangle's x and y
        r.draw_rectangle_pro(
            Rectangle {
                x: ent.x + ent.width / 2.0,
                y: ent.y + ent.height / 2.0,
                width: ent.width,
                height: ent.height,
            },
            Vector2::new(ent.width / 2.0, ent.height / 2.0),
            deg,
            projectile.entity.body.color,
        );
    }
}

fn draw_torpedoes(
    r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
    entities: &Entities,
    viewport: Rectangle,
    delta: f32,
) {
    for torpedo in &entities.torpedoes {
        let bounds = torpedo.entity.body.polygon.bounds_real.lerp(delta);

        if bounds.cull(viewport) {
            continue;
        }

        let gen = torpedo.entity.body.state;
        let rot = gen.old.rotation.lerp(gen.new.rotation, delta);
        let rad = rot.y.atan2(rot.x);
        let deg = rad.to_degrees();
        let ent = gen.lerp(delta);

        // for some reason we need to add half the width and height to rotated rectangle's x and y
        r.draw_rectangle_pro(
            Rectangle {
                x: ent.x + ent.width / 2.0,
                y: ent.y + ent.height / 2.0,
                width: ent.width,
                height: ent.height,
            },
            Vector2::new(ent.width / 2.0, ent.height / 2.0),
            deg,
            torpedo.entity.body.color,
        );
    }
}

fn draw_explosions(
    r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
    entities: &Entities,
    viewport: Rectangle,
    delta: f32,
) {
    for explosion in &entities.explosions {
        let bounds = explosion.entity.body.polygon.bounds_real.lerp(delta);

        if bounds.cull(viewport) {
            continue;
        }

        let gen = &explosion.entity.body.state;
        let ent = gen.lerp(delta);

        r.draw_pixel_v(ent, explosion.entity.body.color);
    }
}

fn draw_exhausts(
    r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
    entities: &Entities,
    viewport: Rectangle,
    delta: f32,
) {
    for exhaust in &entities.exhausts {
        let bounds = exhaust.entity.body.polygon.bounds_real.lerp(delta);

        if bounds.cull(viewport) {
            continue;
        }

        let gen = &exhaust.entity.body.state;
        let ent = gen.lerp(delta);

        r.draw_pixel_v(ent, exhaust.entity.body.color);
    }
}

fn draw_triships(
    r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>,
    entities: &Entities,
    viewport: Rectangle,
    debug: bool,
    delta: f32,
) {
    for triship in &entities.triships {
        let bounds = triship.entity.body.polygon.bounds_real.lerp(delta);

        if bounds.cull(viewport) {
            continue;
        }

        let gen = &triship.entity.body.state;
        let rot = gen.old.rotation.lerp(gen.new.rotation, delta);
        let rad = rot.y.atan2(rot.x);
        let (sin, cos) = rad.sin_cos();
        let ent = gen.lerp(delta);
        let ori = ent.centroid();
        let ent = Triangle {
            v1: rotate(ent.v1, ori, sin, cos),
            v2: rotate(ent.v2, ori, sin, cos),
            v3: rotate(ent.v3, ori, sin, cos),
        };

        r.draw_triangle_lines(ent.v1, ent.v2, ent.v3, triship.entity.body.color);

        if !debug {
            continue;
        }

        r.draw_pixel_v(ori, Color::BLUE);

        r.draw_text_ex(
            r.get_font_default(),
            &format!("{} {}", rad.to_degrees(), rad),
            ent.v1,
            10.0,
            1.0,
            Color::WHITE,
        );
        r.draw_text_ex(
            r.get_font_default(),
            &format!("{} {}", rot.x, rot.y),
            ent.v1.add(Vector2::new(0.0, -10.0)),
            10.0,
            1.0,
            Color::WHITE,
        );

        let vel = triship.entity.motion.velocity;
        r.draw_text_ex(
            r.get_font_default(),
            &format!("{}, {}", vel.x, vel.y),
            ent.v2,
            10.0,
            1.0,
            Color::WHITE,
        );

        let cen = triship.entity.body.state.new.shape.centroid();
        r.draw_text_ex(
            r.get_font_default(),
            &format!("{}, {}", cen.x, cen.y),
            ent.v3,
            10.0,
            1.0,
            Color::WHITE,
        );

        let life = triship.entity.life;
        r.draw_text_ex(
            r.get_font_default(),
            &format!("{}", life),
            ent.v2.add(Vector2::new(0.0, -10.0)),
            10.0,
            1.0,
            Color::WHITE,
        );

        r.draw_rectangle_lines_ex(bounds, 1.0, Color::BLUE);

        let bounds = &triship.entity.body.polygon.bounds_meld.lerp(delta);
        r.draw_rectangle_lines_ex(bounds, 1.0, Color::BLUE);

        // let top = ori + rot * triship.entity.target.height;
        // let width_half = triship.entity.target.width / 2.0;

        // let v1 = top + Vector2::new(rot.y, -rot.x) * width_half;
        // let v2 = ori;
        // let v3 = top + Vector2::new(-rot.y, rot.x) * width_half;
        // // r.draw_triangle_lines(v1, v2, v3, Color::BLUE);
        // r.draw_line_v(v1, v2, Color::BLUE);
        // r.draw_line_v(v2, v3, Color::BLUE);
        // r.draw_line_v(v3, v1, Color::BLUE);

        let viewport = Rectangle {
            x: ori.x - RENDER_WIDTH as f32 / 2.0 + 50.0,
            y: ori.y - RENDER_HEIGHT as f32 / 2.0 + HUD_HEIGHT as f32 / 2.0 + 50.0,
            width: RENDER_WIDTH as f32 - 100.0,
            height: RENDER_HEIGHT as f32 - HUD_HEIGHT as f32 - 100.0,
        };
        // let viewport = Rectangle {
        //     x: ori.x - 200.0,
        //     y: ori.y - 200.0,
        //     width: 400.0,
        //     height: 400.0,
        // };
        // let viewport = Rectangle {
        //     x: ori.x - 150.0,
        //     y: ori.y - 300.0,
        //     width: 300.0,
        //     height: 600.0,
        // };

        // r.draw_rectangle_lines_ex(viewport, 1.0, Color::BLUE);

        // let (y, x) = (rot.y.atan2(rot.x) - 0.56).sin_cos();
        // let rot_left = Vector2::new(x, y);
        // let red = g(ori, rot_left, viewport);
        // r.draw_line_v(ori, red, Color::RED);

        // let (y, x) = (rot.y.atan2(rot.x) + 0.56).sin_cos();
        // let rot_right = Vector2::new(x, y);
        // let green = g(ori, rot_right, viewport);
        // r.draw_line_v(ori, green, Color::GREEN);

        // // x p[ gr;n, y p[ r;d]]
        // let a = Vector2::new(green.x, red.y);
        // r.draw_line_v(green, a, Color::YELLOW);

        // let b = Vector2::new(red.x, red.y);
        // r.draw_line_v(a, b, Color::PINK);

        // ----

        // let top = ori + rot * ((viewport.width / 3.0) * 2.0);
        // let width_two_thirds = triship.entity.target.width / 2.0;
        // let v1 = top + Vector2::new(rot.y, -rot.x) * width_two_thirds;
        // let v2 = ori;
        // let v3 = top + Vector2::new(-rot.y, rot.x) * width_two_thirds;
        // r.draw_triangle_lines(v1, v2, v3, Color::ORANGE);

        let verts = gx(ori, rot, viewport);
        let c = [
            Color::RED,
            Color::ORANGE,
            Color::GREEN,
            Color::PINK,
            Color::SALMON,
            Color::BROWN,
        ];
        for i in 0..verts.len() {
            let v1 = verts[i];
            let v2 = verts[if i + 1 == verts.len() { 0 } else { i + 1 }];

            r.draw_line_v(v1, v2, c[i]);
        }
    }

    fn gx(ori: Vector2, rot: Vector2, viewport: Rectangle) -> Vec<Vector2> {
        let mut v = Vec::new();
        let angle = 0.56;
        let rad = rot.y.atan2(rot.x);

        let (y, x) = (rad - angle).sin_cos();
        let a = g(ori, Vector2::new(x, y), viewport);

        let (y, x) = (rad + angle).sin_cos();
        let b = g(ori, Vector2::new(x, y), viewport);

        v.push(ori);
        v.push(a);
        v.push(b);

        let epsilon = 0.5;
        let equal = |one: f32, two: f32| (one - two).abs() < epsilon;
        let between = |one, two, length| one > two && one < two + length;
        let closed = |one: Vector2, two: Vector2| {
            if equal(one.x, two.x) {
                // cannot be closed if not by the edge
                if equal(one.x, viewport.x) || equal(one.x, viewport.x + viewport.width) {
                    return true;
                }
            }

            if equal(one.y, two.y) {
                // cannot be closed if not by the edge
                if equal(one.y, viewport.y) || equal(one.y, viewport.y + viewport.height) {
                    return true;
                }
            }

            false
        };
        let mut open = !closed(a, b);

        // add new vertexes until we have a closed polygon
        while open {
            let prev = v[v.len() - 2];

            let c = if equal(prev.x, viewport.x) && between(prev.y, viewport.y, viewport.height) {
                Vector2::new(viewport.x, viewport.y)
            } else if equal(prev.x, viewport.x + viewport.width)
                && between(prev.y, viewport.y, viewport.height)
            {
                Vector2::new(viewport.x + viewport.width, viewport.y + viewport.height)
            } else if equal(prev.y, viewport.y) && between(prev.x, viewport.x, viewport.width) {
                Vector2::new(viewport.x + viewport.width, viewport.y)
            } else if equal(prev.y, viewport.y + viewport.height)
                && between(prev.x, viewport.x, viewport.width)
            {
                Vector2::new(viewport.x, viewport.y + viewport.height)
            } else {
                if prev.x == viewport.x && prev.y == viewport.y {
                    Vector2::new(viewport.x + viewport.width, viewport.y)
                } else if prev.x == viewport.x + viewport.width && prev.y == viewport.y {
                    Vector2::new(viewport.x + viewport.width, viewport.y + viewport.height)
                } else if prev.x == viewport.x && prev.y == viewport.y + viewport.height {
                    Vector2::new(viewport.x, viewport.y)
                } else {
                    Vector2::new(viewport.x, viewport.y + viewport.height)
                }
            };

            v.insert(v.len() - 1, c);

            open = !closed(c, b);
        }

        v
    }

    fn g(a: Vector2, direction: Vector2, viewport: Rectangle) -> Vector2 {
        let b = a + direction * 10000.0;

        if let Some(x) = intersection(
            a,
            b,
            Vector2::new(viewport.x, viewport.y),
            Vector2::new(viewport.x + viewport.width, viewport.y),
        ) {
            return x;
        }

        if let Some(x) = intersection(
            a,
            b,
            Vector2::new(viewport.x + viewport.width, viewport.y),
            Vector2::new(viewport.x + viewport.width, viewport.y + viewport.height),
        ) {
            return x;
        }

        if let Some(x) = intersection(
            a,
            b,
            Vector2::new(viewport.x + viewport.width, viewport.y + viewport.height),
            Vector2::new(viewport.x, viewport.y + viewport.height),
        ) {
            return x;
        }

        if let Some(x) = intersection(
            a,
            b,
            Vector2::new(viewport.x, viewport.y + viewport.height),
            Vector2::new(viewport.x, viewport.y),
        ) {
            return x;
        }

        panic!("wtf big viewport");
    }
}
