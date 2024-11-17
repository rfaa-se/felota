use raylib::prelude::*;

use crate::constants::{
    MINIMAP_AREA_HEIGHT, MINIMAP_AREA_WIDTH, MINIMAP_HEIGHT, MINIMAP_WIDTH, MINIMAP_X, MINIMAP_Y,
    TARGETING_AREA_HEIGHT, TARGETING_AREA_WIDTH,
};

pub fn generate_targeting_area(centroid: Vector2) -> Rectangle {
    Rectangle {
        x: centroid.x - (TARGETING_AREA_WIDTH / 2) as f32,
        y: centroid.y - (TARGETING_AREA_HEIGHT / 2) as f32,
        width: TARGETING_AREA_WIDTH as f32,
        height: TARGETING_AREA_HEIGHT as f32,
    }
}

pub fn minimap_translate(x: f32, y: f32, minimap: Vector2) -> Vector2 {
    let mah = MINIMAP_AREA_HEIGHT as f32;
    let mh = MINIMAP_HEIGHT as f32;
    let maw = MINIMAP_AREA_WIDTH as f32;
    let mw = MINIMAP_WIDTH as f32;

    let rh = mh / mah;
    let rw = mw / maw;

    let mh2 = mh / 2.0;
    let mw2 = mw / 2.0;

    let x = x - minimap.x;
    let y = y - minimap.y;

    let x = x + mw2;
    let y = y + mh2;

    let x = x * rw + MINIMAP_X as f32;
    let y = y * rh + MINIMAP_Y as f32;

    let x = x + mw2;
    let y = y + mh2;

    Vector2::new(x, y)
}
