use raylib::prelude::*;

use crate::constants::{TARGETING_AREA_HEIGHT, TARGETING_AREA_WIDTH};

pub fn generate_targeting_area(centroid: Vector2) -> Rectangle {
    Rectangle {
        x: centroid.x - (TARGETING_AREA_WIDTH / 2) as f32,
        y: centroid.y - (TARGETING_AREA_HEIGHT / 2) as f32,
        width: TARGETING_AREA_WIDTH as f32,
        height: TARGETING_AREA_HEIGHT as f32,
    }
}
