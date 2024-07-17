use raylib::prelude::*;

use crate::{
    components::{Centroidable, Lerpable, Rotatable},
    constants::{COSMOS_HEIGHT, COSMOS_WIDTH},
    entities::Entities,
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
        delta: f32,
    ) {
        for triship in &entities.triships {
            let gen = &triship.entity.body.generation;
            let rot = gen.old.rotation.lerp(gen.new.rotation, delta);
            let ent = gen.lerp(delta).rotated(rot);

            r.draw_triangle_lines(ent.v1, ent.v2, ent.v3, triship.entity.body.color);

            // let deg = gen.new.rotation.y.atan2(gen.new.rotation.x).to_degrees();
            // r.draw_text_ex(
            //     r.get_font_default(),
            //     &format!("{}", deg),
            //     ent.v1,
            //     10.0,
            //     1.0,
            //     Color::WHITE,
            // );

            // let vel = triship.entity.motion.velocity;
            // r.draw_text_ex(
            //     r.get_font_default(),
            //     &format!("{}, {}", vel.x, vel.y),
            //     ent.v2,
            //     10.0,
            //     1.0,
            //     Color::WHITE,
            // );

            // let cen = triship.entity.body.generation.new.shape.centroid();
            // r.draw_text_ex(
            //     r.get_font_default(),
            //     &format!("{}, {}", cen.x, cen.y),
            //     ent.v3,
            //     10.0,
            //     1.0,
            //     Color::WHITE,
            // );
        }

        for exhaust in &entities.exhausts {
            let gen = &exhaust.entity.body.generation;
            let ent = gen.lerp(delta);

            r.draw_pixel_v(ent, exhaust.entity.body.color);
        }

        r.draw_rectangle_lines_ex(viewport, 1.0, Color::RED);

        r.draw_rectangle_lines(0, 0, COSMOS_WIDTH, COSMOS_HEIGHT, Color::RED);
    }
}
