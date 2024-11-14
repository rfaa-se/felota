use std::ops::Add;

use crate::{
    components::*,
    constants::{STARFIELD_HEIGHT, STARFIELD_WIDTH},
    entities::*,
};

use raylib::prelude::*;

pub struct Forge {}

impl Forge {
    pub fn new() -> Self {
        Self {}
    }

    pub fn triship(&self, position: Vector2) -> Triship {
        let d = Direction::SOUTHEAST;
        let w = 50.0;
        let w3 = w / 3.0;
        let h = 60.0;
        let h3 = h / 3.0;
        let s = RotatedShape {
            shape: Triangle {
                // v1: Vector2::new(50.0, 50.0),
                // v2: Vector2::new(110.0, 75.0),
                // v3: Vector2::new(50.0, 100.0),
                v1: Vector2::new(position.x - w3, position.y - h3),
                v2: Vector2::new(position.x + w3 * 2.0, position.y),
                v3: Vector2::new(position.x - w3, position.y + h3),
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
            life: 100.0,
            body: Body {
                state: Generation { old: s, new: s },
                color: Color::DIMGRAY,
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
                rotation_acceleration: 0.016,
                rotation_speed_max: 0.28,
            },
            boost: Boost {
                acceleration: 1.6,
                acceleration_old: 0.0,
                speed_max: 40.0,
                speed_max_old: 0.0,
                lifetime: Load {
                    current: 0,
                    max: 100,
                },
                cooldown: Load {
                    current: 0,
                    max: 50,
                },
                active: false,
            },
            cooldown_torpedo: Load {
                current: 0,
                max: 20,
            },
            targeting: Targeting {
                eid: None,
                timer: Load {
                    current: 0,
                    max: 50,
                },
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
                speed_max: 30.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.0,
                rotation_speed_max: 0.0,
            },
            owner_id,
            life: 1.0,
        }
    }

    pub fn torpedo(
        &self,
        position: Vector2,
        direction: Vector2,
        initial_velocity: Vector2,
        owner_id: usize,
        target: Option<usize>,
    ) -> Torpedo {
        let width = 8.0;
        let height = 3.0;
        let distance = direction * width;
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
        let speed = 8.0;

        // we want the torp to be launched sideways,
        // then once the timer_inactive is 0 it will start
        // accelerating in the requested direction
        let direction = Vector2::new(direction.y, direction.x * -1.0);

        Torpedo {
            damage: 10.0,
            body: Body {
                state: Generation { old: s, new: s },
                color: Color::GRAY,
                polygon: Polygon {
                    dirty: false,
                    vertexes: v_gen,
                    bounds_real: b_gen,
                    bounds_meld: b_gen,
                },
            },
            motion: Motion {
                velocity: initial_velocity + direction * speed,
                acceleration: 1.02, //1.16,
                speed_max: 32.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.04,
                rotation_speed_max: 0.16,
            },
            owner_id,
            timer_inactive: 3,
            life: 1.0,
            target,
        }
    }

    pub fn explosion(
        &self,
        position: Vector2,
        rotation: Vector2,
        lifetime: u8,
        velocity: Vector2,
        acceleration: f32,
        color: Color,
        random: u8,
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
            random,
            body: Body {
                state: Generation { old: s, new: s },
                color,
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
        let amount = 4;
        let mut explosion = Vec::new();
        explosion.reserve_exact(amount);

        for _ in 0..amount {
            let rotation = Vector2::zero();
            let lifetime = h.get_random_value::<i32>(5..20) as u8;
            let x = h.get_random_value::<i32>(-200..200) as f32 / 100.0;
            let y = h.get_random_value::<i32>(-200..200) as f32 / 100.0;
            let velocity = Vector2::new(x, y);
            let acceleration = h.get_random_value::<i32>(1..10) as f32;
            let color = explosion_color(h);
            let random = h.get_random_value::<i32>(1..10) as u8;

            explosion.push(self.explosion(
                position,
                rotation,
                lifetime,
                velocity,
                acceleration,
                color,
                random,
            ));
        }

        explosion
    }

    pub fn explosion_torpedo(&self, position: Vector2, h: &mut RaylibHandle) -> Vec<Particle> {
        let amount = 32;
        let mut explosion = Vec::new();
        explosion.reserve_exact(amount);

        for _ in 0..amount {
            let rotation = Vector2::zero();
            let lifetime = h.get_random_value::<i32>(5..20) as u8;
            let x = h.get_random_value::<i32>(-200..200) as f32 / 100.0;
            let y = h.get_random_value::<i32>(-200..200) as f32 / 100.0;
            let velocity = Vector2::new(x, y);
            let acceleration = h.get_random_value::<i32>(1..10) as f32;
            let color = explosion_color(h);
            let random = h.get_random_value::<i32>(1..15) as u8;

            explosion.push(self.explosion(
                position,
                rotation,
                lifetime,
                velocity,
                acceleration,
                color,
                random,
            ));
        }

        explosion
    }

    pub fn explosion_triship(&self, position: Vector2, h: &mut RaylibHandle) -> Vec<Particle> {
        let amount = 64;
        let mut explosion = Vec::new();
        explosion.reserve_exact(amount);

        for _ in 0..amount {
            let rotation = Vector2::zero();
            let lifetime = h.get_random_value::<i32>(10..30) as u8;
            let x = h.get_random_value::<i32>(-3000..3000) as f32 / 1000.0;
            let y = h.get_random_value::<i32>(-3000..3000) as f32 / 1000.0;
            let velocity = Vector2::new(x, y);
            let acceleration = h.get_random_value::<i32>(1..2000) as f32 / 100.0;
            let color = explosion_color(h);
            let random = h.get_random_value::<i32>(1..15) as u8;

            explosion.push(self.explosion(
                position,
                rotation,
                lifetime,
                velocity,
                acceleration,
                color,
                random,
            ));
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
        random: u8,
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
            random,
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
                let lifetime = (h.get_random_value::<i32>(0..2) + j) as u8;
                let speed = h.get_random_value::<i32>(2..10) as f32;
                let velocity = initial_velocity + rotation * speed;
                let acceleration = h.get_random_value::<i32>(1..4) as f32;
                let random = h.get_random_value::<i32>(10..20) as u8;

                exhaust.push(self.exhaust(pos, rotation, lifetime, velocity, acceleration, random));
            }
        }

        exhaust
    }

    pub fn exhaust_torpedo(
        &self,
        position: Vector2,
        rotation: Vector2,
        initial_velocity: Vector2,
        h: &mut RaylibHandle,
    ) -> Vec<Particle> {
        let mut exhaust = Vec::new();
        exhaust.reserve_exact(3);

        let v = [1, 2, 1];
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
                let random = h.get_random_value::<i32>(10..20) as u8;

                exhaust.push(self.exhaust(pos, rotation, lifetime, velocity, acceleration, random));
            }
        }

        exhaust
    }

    pub fn exhaust_thruster(
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
                let lifetime = (h.get_random_value::<i32>(0..2) + j) as u8;
                let speed = h.get_random_value::<i32>(1..8) as f32;
                let velocity = initial_velocity + rotation * speed;
                let acceleration = h.get_random_value::<i32>(1..4) as f32;
                let random = h.get_random_value::<i32>(10..20) as u8;

                exhaust.push(self.exhaust(pos, rotation, lifetime, velocity, acceleration, random));
            }
        }

        exhaust
    }

    pub fn exhaust_thruster_bow(
        &self,
        position_port: Vector2,
        position_starboard: Vector2,
        rotation: Vector2,
        initial_velocity: Vector2,
        h: &mut RaylibHandle,
    ) -> Vec<Particle> {
        let mut exhaust = Vec::new();
        exhaust.reserve_exact(10);

        exhaust.append(&mut self.exhaust_thruster(position_port, rotation, initial_velocity, h));

        exhaust.append(&mut self.exhaust_thruster(
            position_starboard,
            rotation,
            initial_velocity,
            h,
        ));

        exhaust
    }

    pub fn star(
        &self,
        position: Vector2,
        rotation: Vector2,
        lifetime: u8,
        random: u8,
        velocity: Vector2,
        acceleration: f32,
        color: Color,
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
            random,
            body: Body {
                state: Generation { old: s, new: s },
                color,
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
                speed_max: 5.0,
                rotation_speed: 0.0,
                rotation_acceleration: 0.0,
                rotation_speed_max: 0.0,
            },
        }
    }

    pub fn stars(&self, h: &mut RaylibHandle) -> Vec<Particle> {
        let amount = 128;
        let mut stars = Vec::new();
        stars.reserve_exact(amount);

        for _ in 0..amount {
            let rotation = Vector2::zero();
            let position = Vector2::new(
                h.get_random_value::<i32>(1..STARFIELD_WIDTH - 1) as f32,
                h.get_random_value::<i32>(1..STARFIELD_HEIGHT - 1) as f32,
            );
            let lifetime = 0;
            // 1 in 9 will be moving slightly
            let velocity = if h.get_random_value::<i32>(0..8) > 7 {
                Vector2::new(
                    h.get_random_value::<i32>(-50..50) as f32 / 1000.0,
                    h.get_random_value::<i32>(-50..50) as f32 / 1000.0,
                )
            } else {
                Vector2::zero()
            };

            let acceleration = 0.0;
            let color = Color::new(
                h.get_random_value::<i32>(100..255) as u8,
                h.get_random_value::<i32>(200..255) as u8,
                h.get_random_value::<i32>(200..255) as u8,
                h.get_random_value::<i32>(0..255) as u8,
            );
            let random =
                ((h.get_random_value::<i32>(0..7) << 1) + h.get_random_value::<i32>(0..1)) as u8;

            stars.push(self.star(
                position,
                rotation,
                lifetime,
                random,
                velocity,
                acceleration,
                color,
            ));
        }

        stars
    }
}

fn explosion_color(h: &mut RaylibHandle) -> Color {
    Color {
        r: h.get_random_value::<i32>(250..255) as u8,
        g: h.get_random_value::<i32>(0..8) as u8,
        b: h.get_random_value::<i32>(0..0) as u8,
        a: h.get_random_value::<i32>(100..200) as u8,
    }
}
