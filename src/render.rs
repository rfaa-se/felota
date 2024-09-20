use std::ops::Add;

use raylib::prelude::*;

use crate::{
    components::{Centroidable, Cullable, Lerpable, Triangle},
    constants::{COSMOS_HEIGHT, COSMOS_WIDTH, STARFIELD_HEIGHT, STARFIELD_WIDTH},
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
        draw_triships(r, entities, viewport, debug, delta);
        draw_exhausts(r, entities, viewport, delta);
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

        let deg = gen.new.rotation.y.atan2(gen.new.rotation.x).to_degrees();
        r.draw_text_ex(
            r.get_font_default(),
            &format!("{}", deg),
            ent.v1,
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
    }
}
