use std::ops::Add;

use crate::{components::*, entities::*};

use raylib::prelude::*;

pub struct Forge {}

impl Forge {
    pub fn new() -> Self {
        Self {}
    }

    pub fn triship(&self) -> Triship {
        let d = Direction::SOUTHEAST;
        let s = RotatedShape {
            shape: Triangle {
                v1: Vector2::new(50.0, 50.0),
                v2: Vector2::new(110.0, 75.0),
                v3: Vector2::new(50.0, 100.0),
            },
            rotation: d,
        };
        let v = s.shape.vertexes(d);
        let b = v.bounds();
        let v_gen = Generation {
            old: v.clone(),
            new: v,
        };
        let b_gen = Generation { old: b, new: b };

        Triship {
            life: 10.0,
            body: Body {
                state: Generation { old: s, new: s },
                color: Color::RED,
                polygon: Polygon {
                    dirty: false,
                    vertexes: v_gen,
                    bounds_real: b_gen,
                    bounds_meld: b_gen,
                },
            },
            motion: Motion {
                velocity: Vector2::zero(),
                speed_max: 20.0,
                acceleration: 1.02,
                rotation_speed: 0.0,
                rotation_acceleration: 0.02,
                rotation_speed_max: 0.24,
            },
            boost: Boost {
                acceleration: 1.6,
                acceleration_old: 0.0,
                speed_max: 40.0,
                speed_max_old: 0.0,
                lifetime: 0,
                lifetime_max: 100,
                cooldown: 0,
                cooldown_max: 50,
                active: false,
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
        let p = Vector2::new(position.x + distance.x, position.y + distance.y);
        let s = RotatedShape {
            shape: Rectangle {
                x: p.x - width / 2.0,
                y: p.y - height / 2.0,
                width,
                height,
            },
            rotation: direction,
        };
        let v = s.shape.vertexes(direction);
        let b = v.bounds();
        let v_gen = Generation {
            old: v.clone(),
            new: v,
        };
        let b_gen = Generation { old: b, new: b };
        let speed = 30.0;

        Projectile {
            damage: 2.0,
            body: Body {
                state: Generation { old: s, new: s },
                color: Color::LIGHTGOLDENRODYELLOW,
                polygon: Polygon {
                    dirty: false,
                    vertexes: v_gen,
                    bounds_real: b_gen,
                    bounds_meld: b_gen,
                },
            },
            motion: Motion {
                velocity: initial_velocity + direction * speed,
                acceleration: 1.1,
                speed_max: 40.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.0,
                rotation_speed_max: 0.0,
            },
            owner_id,
        }
    }

    pub fn explosion(
        &self,
        position: Vector2,
        rotation: Vector2,
        lifetime: u8,
        velocity: Vector2,
        acceleration: f32,
    ) -> Particle {
        let s = RotatedShape {
            shape: position,
            rotation,
        };
        let v = s.shape.vertexes(rotation);
        let b = v.bounds();
        let v_gen = Generation {
            old: v.clone(),
            new: v,
        };
        let b_gen = Generation { old: b, new: b };

        Particle {
            lifetime,
            body: Body {
                state: Generation { old: s, new: s },
                color: Color::WHITE,
                polygon: Polygon {
                    dirty: false,
                    vertexes: v_gen,
                    bounds_real: b_gen,
                    bounds_meld: b_gen,
                },
            },
            motion: Motion {
                velocity,
                acceleration,
                speed_max: 25.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.0,
                rotation_speed_max: 0.0,
            },
        }
    }

    pub fn explosion_projectile(&self, position: Vector2, h: &mut RaylibHandle) -> Vec<Particle> {
        let amount = 16;
        let mut explosion = Vec::new();
        explosion.reserve_exact(amount);

        for _ in 0..amount {
            let rotation = Vector2::zero();
            let lifetime = h.get_random_value::<i32>(5..20) as u8;
            let x = h.get_random_value::<i32>(-200..200) as f32 / 100.0;
            let y = h.get_random_value::<i32>(-200..200) as f32 / 100.0;
            let velocity = Vector2::new(x, y);
            let acceleration = h.get_random_value::<i32>(1..10) as f32;

            explosion.push(self.explosion(position, rotation, lifetime, velocity, acceleration));
        }

        explosion
    }

    pub fn exhaust(
        &self,
        position: Vector2,
        rotation: Vector2,
        lifetime: u8,
        velocity: Vector2,
        acceleration: f32,
    ) -> Particle {
        let s = RotatedShape {
            shape: position,
            rotation,
        };
        let v = s.shape.vertexes(rotation);
        let b = v.bounds();
        let v_gen = Generation {
            old: v.clone(),
            new: v,
        };
        let b_gen = Generation { old: b, new: b };

        Particle {
            lifetime,
            body: Body {
                state: Generation { old: s, new: s },
                color: Color::LIGHTSKYBLUE,
                polygon: Polygon {
                    dirty: false,
                    vertexes: v_gen,
                    bounds_real: b_gen,
                    bounds_meld: b_gen,
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
        exhaust.reserve_exact(32);

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
                let speed = h.get_random_value::<i32>(2..10) as f32;
                let velocity = initial_velocity + rotation * speed;
                let acceleration = h.get_random_value::<i32>(1..4) as f32;
                // let velocity = velocity.clamp(-20.0..20.0);
                // println!("{:?}", velocity);

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
        exhaust.reserve_exact(5);

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
                let acceleration = h.get_random_value::<i32>(1..4) as f32;

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
