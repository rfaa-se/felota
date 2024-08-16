use std::ops::Add;

use crate::{components::*, entities::*};

use raylib::prelude::*;

pub struct Forge {}

impl Forge {
    pub fn new() -> Self {
        Self {}
    }

    pub fn triship(&self) -> Triship {
        let d = Direction::NORTH;
        let shape = RotatedShape {
            shape: Triangle {
                v1: Vector2::new(50.0, 50.0),
                v2: Vector2::new(110.0, 75.0),
                v3: Vector2::new(50.0, 100.0),
            },
            rotation: d,
        };

        Triship {
            life: 10.0,
            body: Body {
                generation: Generation {
                    old: shape,
                    new: shape,
                },
                color: Color::RED,
                polygon: Polygon {
                    dirty: true,
                    vertexes: Vec::new(),
                    bounds: Rectangle::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            motion: Motion {
                velocity: Vector2::zero(),
                speed_max: 20.0,
                acceleration: 1.02,
                rotation_speed: 0.0,
                rotation_acceleration: 0.04,
                rotation_speed_max: 0.24,
            },
        }
    }

    pub fn projectile(
        &self,
        position: Vector2,
        direction: Vector2,
        initial_velocity: Vector2,
        owner_id: usize,
    ) -> Projectile {
        // |\
        // | \
        // |  \_ <- placement will be here, in front of ship
        // |  /
        // | /
        // |/

        let width = 2.0;
        let height = 1.0;
        let distance = direction * (width / 2.0);
        let position = Vector2::new(position.x + distance.x, position.y + distance.y);
        let shape = RotatedShape {
            shape: Rectangle {
                x: position.x - width / 2.0,
                y: position.y - height / 2.0,
                width,
                height,
            },
            rotation: direction,
        };

        let speed = 20.0;

        Projectile {
            damage: 2.0,
            body: Body {
                generation: Generation {
                    old: shape,
                    new: shape,
                },
                color: Color::LIGHTGOLDENRODYELLOW,
                polygon: Polygon {
                    dirty: true,
                    vertexes: Vec::new(),
                    bounds: Rectangle::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            motion: Motion {
                velocity: initial_velocity + direction * speed,
                acceleration: 1.1,
                speed_max: 30.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.0,
                rotation_speed_max: 0.0,
            },
            owner_id,
        }
    }

    pub fn exhaust(
        &self,
        position: Vector2,
        rotation: Vector2,
        lifetime: u8,
        velocity: Vector2,
        acceleration: f32,
    ) -> Particle {
        let shape = RotatedShape {
            shape: position,
            rotation,
        };

        Particle {
            lifetime,
            body: Body {
                generation: Generation {
                    old: shape,
                    new: shape,
                },
                color: Color::LIGHTSKYBLUE,
                polygon: Polygon {
                    dirty: true,
                    vertexes: Vec::new(),
                    bounds: Rectangle::new(0.0, 0.0, 0.0, 0.0),
                },
            },
            motion: Motion {
                velocity,
                acceleration,
                speed_max: 20.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.0,
                rotation_speed_max: 0.0,
            },
        }
    }

    pub fn exhaust_afterburner(
        &self,
        position: Vector2,
        rotation: Vector2,
        initial_velocity: Vector2,
        h: &mut RaylibHandle,
    ) -> Vec<Particle> {
        // 32 particles
        // 0 1 2 3 4 5 6
        // 2 4 6 8 6 4 2
        // x is the [position] received
        // . . . x . . .
        // . . . . . . .
        //   . . . . .
        //   . . . . .
        //     . . .
        //     . . .
        //       .
        //       .

        let mut exhaust = Vec::new();
        let v = [2, 4, 6, 8, 6, 4, 2];
        let neg_half = -((v.len() / 2) as f32);

        // rotate by 90 degrees so we can start placing the particles in a line
        let rot = Vector2 {
            x: rotation.y,
            y: rotation.x * -1.0,
        };

        for i in 0..v.len() {
            for j in 0..v[i] {
                let pos = Vector2 {
                    x: neg_half + i as f32,
                    y: 0.0,
                }
                .rotated(rot.y.atan2(rot.x))
                .add(position);

                // some random values to make it look awesome
                let lifetime = (h.get_random_value::<i32>(0..4) + j) as u8;
                let speed = h.get_random_value::<i32>(1..6) as f32;
                let velocity = initial_velocity + rotation * speed;
                let acceleration = h.get_random_value::<i32>(1..4) as f32 / speed;

                // clamp shit!? once ship is heading in a straight line, the afterburner gets smaller

                exhaust.push(self.exhaust(pos, rotation, lifetime, velocity, acceleration));
            }
        }

        exhaust
    }

    pub fn exhaust_thruster_side(
        &self,
        position: Vector2,
        rotation: Vector2,
        initial_velocity: Vector2,
        h: &mut RaylibHandle,
    ) -> Vec<Particle> {
        let mut exhaust = Vec::new();
        let v = [1, 3, 1];
        let neg_half = -((v.len() / 2) as f32);

        // rotate by 90 degrees so we can start placing the particles in a line
        let rot = Vector2 {
            x: rotation.y,
            y: rotation.x * -1.0,
        };

        for i in 0..v.len() {
            for j in 0..v[i] {
                let pos = Vector2 {
                    x: neg_half + i as f32,
                    y: 0.0,
                }
                .rotated(rot.y.atan2(rot.x))
                .add(position);

                // some random values to make it look awesome
                let lifetime = (h.get_random_value::<i32>(0..4) + j) as u8;
                let speed = h.get_random_value::<i32>(1..6) as f32;
                let velocity = initial_velocity + rotation * speed;
                let acceleration = h.get_random_value::<i32>(1..4) as f32 / speed;

                exhaust.push(self.exhaust(pos, rotation, lifetime, velocity, acceleration));
            }
        }

        exhaust
    }

    pub fn _exhaust_thruster_bow(
        &self,
        position_port: Vector2,
        position_starboard: Vector2,
        initial_velocity: Vector2,
        rotation: Vector2,
        h: &mut RaylibHandle,
    ) -> Vec<Particle> {
        // TODO
        let mut exhaust = Vec::new();

        // let originator_velocity = originator_velocity * -1.0;

        exhaust.append(&mut self.exhaust_thruster_side(
            position_port,
            rotation,
            initial_velocity,
            h,
        ));

        exhaust.append(&mut self.exhaust_thruster_side(
            position_starboard,
            rotation,
            initial_velocity,
            h,
        ));

        exhaust
    }
}
