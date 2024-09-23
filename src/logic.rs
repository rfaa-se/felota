mod collisions;

use std::collections::BTreeSet;

use crate::{
    bus::Bus,
    commands::{Command, EntityCommands},
    components::{Generationable, Motion, Renewable, Shape},
    constants::{COSMOS_HEIGHT, COSMOS_WIDTH, STARFIELD_HEIGHT, STARFIELD_WIDTH},
    entities::{Entities, EntityIndex},
    forge::Forge,
    messages::LogicMessage,
    quadtree::QuadTree,
};

use raylib::prelude::*;

use collisions::*;

const COSMIC_DRAG: Vector2 = Vector2::new(0.1, 0.1);
const COSMIC_DRAG_ROTATION: f32 = 0.002;

pub struct Logic {
    dead: BTreeSet<usize>,
    collisions: Vec<(EntityIndex, EntityIndex)>,
    quadtree: QuadTree,
    commands: Vec<(usize, Command)>,
}

impl Logic {
    pub fn new() -> Self {
        Self {
            dead: BTreeSet::new(),
            collisions: Vec::new(),
            quadtree: QuadTree::new(COSMOS_WIDTH, COSMOS_HEIGHT),
            commands: Vec::new(),
        }
    }

    pub fn update(
        &mut self,
        bus: &mut Bus,
        entities: &mut Entities,
        entity_cmds: &[EntityCommands],
        forge: &Forge,
        h: &mut RaylibHandle,
    ) {
        update_dead_removal(entities, &mut self.dead);
        update_body_generation(entities);
        update_commands(entities, entity_cmds, &mut self.commands, forge, h);
        update_boost(entities);
        update_cooldowns(entities);
        update_motion(entities);
        update_body(entities);
        update_collision_detection(entities, &mut self.quadtree, &mut self.collisions);
        update_collision_reaction(entities, &mut self.collisions, forge, h);
        update_particles_exhaust_alpha(entities);
        update_particles_lifetime(entities, &mut self.dead);
        update_particles_explosions(entities);
        update_particles_stars(entities);
        update_torpedo_timers(entities);
        update_commands_accelerate(entities, &mut self.commands);
        update_out_of_bounds(entities, &mut self.dead);
        update_dead_detection(entities, &mut self.dead);
        update_dead_notify(entities, &self.dead, bus);
    }

    pub fn draw(&self, r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>) {
        self.quadtree.draw(r);
    }
}

// TODO: move these functions into their own files? need to figure out structure

fn update_torpedo_timers(entities: &mut Entities) {
    entities
        .torpedoes
        .iter_mut()
        .filter_map(|x| {
            if x.entity.timer_inactive != 0 {
                Some(&mut x.entity.timer_inactive)
            } else {
                None
            }
        })
        .for_each(|x| {
            *x -= 1;
        })
}

fn update_commands_accelerate(entities: &mut Entities, commands: &mut Vec<(usize, Command)>) {
    entities
        .torpedoes
        .iter()
        .filter_map(|x| {
            if x.entity.life > 0.0 {
                Some(x.id)
            } else {
                None
            }
        })
        .for_each(|x| {
            commands.push((x, Command::Accelerate));
        });
}

fn update_cooldowns(entities: &mut Entities) {
    entities
        .triships
        .iter_mut()
        .filter_map(|x| {
            if x.entity.cooldown_torpedo.current != 0 {
                Some(&mut x.entity.cooldown_torpedo.current)
            } else {
                None
            }
        })
        .for_each(|x| *x -= 1);
}

fn update_particles_stars(entities: &mut Entities) {
    entities.stars.iter_mut().for_each(|x| {
        // 0b_0000_0000
        //            x: add, bool
        //         xxx : amount, 0-7

        let c = &mut x.entity.body.color;
        let r = &mut x.entity.random;
        let add = *r << 7 >> 7 > 0;
        let amount = *r << 4 >> 5;

        if amount == 0 {
            return;
        }

        // twinkle twinkle little star
        if add {
            if c.a >= u8::MAX - amount {
                // if we cannot add anymore, time to toggle
                *r -= 1;
            } else {
                c.a += amount;
            }
        } else {
            if c.a <= amount {
                // if we cannot subtract anymore, time to toggle
                *r += 1;
            } else {
                c.a -= amount;
            }
        }

        let s = &mut x.entity.body.state.new.shape;
        let mut regen = false;

        if s.x < 0.0 {
            s.x += STARFIELD_WIDTH as f32;
            regen = true;
        }

        if s.x > STARFIELD_WIDTH as f32 {
            s.x -= STARFIELD_WIDTH as f32;
            regen = true;
        }

        if s.y < 0.0 {
            s.y += STARFIELD_HEIGHT as f32;
            regen = true;
        }

        if s.y > STARFIELD_HEIGHT as f32 {
            s.y -= STARFIELD_HEIGHT as f32;
            regen = true;
        }

        if regen {
            x.entity.body.state.generation();
            x.entity.body.polygon.dirty = true;
            x.entity.body.renew();
        }
    });
}

fn update_particles_explosions(entities: &mut Entities) {
    entities.explosions.iter_mut().for_each(|x| {
        let r = x.entity.random;
        let c = &mut x.entity.body.color;

        if c.g < u8::MAX - r {
            c.g += r;
        } else {
            c.g = u8::MAX;
        }

        if c.a > r {
            c.a -= r;
        } else {
            c.a = 0;
        }
    });
}

fn update_dead_notify(entities: &mut Entities, dead: &BTreeSet<usize>, bus: &mut Bus) {
    for eid in dead {
        if let Some(eidx) = entities.entity(*eid) {
            bus.send(LogicMessage::EntityDead(*eid, eidx));
        }
    }
}

fn update_dead_detection(entities: &mut Entities, dead: &mut BTreeSet<usize>) {
    entities
        .triships
        .iter()
        .map(|x| (x.id, x.entity.life))
        .chain(entities.projectiles.iter().map(|x| (x.id, x.entity.life)))
        .chain(entities.torpedoes.iter().map(|x| (x.id, x.entity.life)))
        .filter_map(|(id, life)| if life <= 0.0 { Some(id) } else { None })
        .for_each(|id| {
            dead.insert(id);
        });
}

fn update_boost(entities: &mut Entities) {
    entities
        .triships
        .iter_mut()
        .map(|x| (&mut x.entity.motion, &mut x.entity.boost))
        .filter(|(_, boost)| boost.active)
        .for_each(|(motion, boost)| {
            if boost.lifetime.current == 0 {
                boost.cooldown.current -= 1;

                if boost.cooldown.current != 0 {
                    return;
                }

                // cooldown is done, boost is ready
                boost.cooldown.current = boost.cooldown.max;
                boost.lifetime.current = boost.lifetime.max;
                boost.active = false;
            } else {
                boost.lifetime.current -= 1;

                if boost.lifetime.current != 0 {
                    return;
                }

                // reset old values, boost has been exhausted
                motion.speed_max = boost.speed_max_old;
                motion.acceleration = boost.acceleration_old;
            }
        });
}

fn update_particles_exhaust_alpha(entities: &mut Entities) {
    entities.exhausts.iter_mut().for_each(|x| {
        let c = &mut x.entity.body.color;
        let r = x.entity.random;

        if c.a > 0 + r {
            c.a -= r;
        } else {
            c.a = 0;
        }
    });
}

fn update_particles_lifetime(entities: &mut Entities, dead: &mut BTreeSet<usize>) {
    entities
        .exhausts
        .iter_mut()
        .chain(entities.explosions.iter_mut())
        .for_each(|x| {
            if x.entity.lifetime == 0 {
                dead.insert(x.id);
            } else {
                x.entity.lifetime -= 1;
            }
        })
}

fn update_dead_removal(entities: &mut Entities, dead: &mut BTreeSet<usize>) {
    while let Some(d) = dead.pop_first() {
        entities.kill(d);
    }
}

fn update_body_generation(entities: &mut Entities) {
    entities
        .triships
        .iter_mut()
        .map(|x| &mut x.entity.body as &mut dyn Generationable)
        .chain(
            entities
                .projectiles
                .iter_mut()
                .map(|x| &mut x.entity.body as &mut dyn Generationable),
        )
        .chain(
            entities
                .exhausts
                .iter_mut()
                .map(|x| &mut x.entity.body as &mut dyn Generationable),
        )
        .chain(
            entities
                .explosions
                .iter_mut()
                .map(|x| &mut x.entity.body as &mut dyn Generationable),
        )
        .chain(
            entities
                .stars
                .iter_mut()
                .map(|x| &mut x.entity.body as &mut dyn Generationable),
        )
        .chain(
            entities
                .torpedoes
                .iter_mut()
                .map(|x| &mut x.entity.body as &mut dyn Generationable),
        )
        .for_each(|body| body.generation());
}

fn update_commands(
    entities: &mut Entities,
    entity_cmds: &[EntityCommands],
    entity_cmds_internal: &mut Vec<(usize, Command)>,
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    for entity_cmd in entity_cmds {
        for cmd in entity_cmd.commands.iter() {
            cmd.execute(entities, entity_cmd.id, forge, h);
        }
    }

    while let Some((id, cmd)) = entity_cmds_internal.pop() {
        cmd.execute(entities, id, forge, h);
    }
}

fn update_motion(entities: &mut Entities) {
    entities
        .triships
        .iter_mut()
        .map(|x| (&mut x.entity.motion, true))
        .chain(
            entities
                .projectiles
                .iter_mut()
                .map(|x| (&mut x.entity.motion, false)),
        )
        .chain(
            entities
                .exhausts
                .iter_mut()
                .map(|x| (&mut x.entity.motion, true)),
        )
        .chain(
            entities
                .explosions
                .iter_mut()
                .map(|x| (&mut x.entity.motion, false)),
        )
        .chain(
            entities
                .stars
                .iter_mut()
                .map(|x| (&mut x.entity.motion, false)),
        )
        .chain(
            entities
                .torpedoes
                .iter_mut()
                .map(|x| (&mut x.entity.motion, false)),
        )
        .for_each(|(motion, apply_drag)| {
            if apply_drag {
                apply_cosmic_drag(motion);
            }

            check_speed_max(motion);
            check_rotation_speed_max(motion);
        });

    fn apply_cosmic_drag(motion: &mut Motion) {
        let norm = motion.velocity.normalized();
        let drag = norm * COSMIC_DRAG;

        motion.velocity -= drag;

        // if entity has suddenly switched direction after the drag, we should make a full stop
        if norm.dot(motion.velocity.normalized()) < 0.0 {
            motion.velocity = Vector2::zero();
        }

        if motion.rotation_speed < 0.0 {
            motion.rotation_speed += COSMIC_DRAG_ROTATION;
            if motion.rotation_speed > 0.0 {
                motion.rotation_speed = 0.0;
            }
        } else if motion.rotation_speed > 0.0 {
            motion.rotation_speed -= COSMIC_DRAG_ROTATION;
            if motion.rotation_speed < 0.0 {
                motion.rotation_speed = 0.0;
            }
        }
    }

    fn check_speed_max(motion: &mut Motion) {
        // since we can boost we don't want to directly set the velocity to the max speed
        // when we run out of boost, the velocity should slowly decrease until we hit max speed
        if motion.velocity.length_sqr() > motion.speed_max.powi(2) {
            let direction = motion.velocity.normalized();

            motion.velocity -= direction * motion.acceleration;

            if motion.velocity.length_sqr() < motion.speed_max.powi(2) {
                motion.velocity = direction * motion.speed_max;
            }
        }
    }

    fn check_rotation_speed_max(motion: &mut Motion) {
        if motion.rotation_speed > motion.rotation_speed_max {
            motion.rotation_speed = motion.rotation_speed_max;
        } else if motion.rotation_speed < -motion.rotation_speed_max {
            motion.rotation_speed = -motion.rotation_speed_max;
        }
    }
}

fn update_body(entities: &mut Entities) {
    entities
        .triships
        .iter_mut()
        .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion))
        .chain(
            entities
                .projectiles
                .iter_mut()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .chain(
            entities
                .exhausts
                .iter_mut()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .chain(
            entities
                .explosions
                .iter_mut()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .chain(
            entities
                .stars
                .iter_mut()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .chain(
            entities
                .torpedoes
                .iter_mut()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .for_each(|(shape, motion)| {
            shape.accelerate(motion.velocity);
            shape.rotate(motion.rotation_speed);
            shape.renew();
        });
}

fn update_out_of_bounds(entities: &mut Entities, dead: &mut BTreeSet<usize>) {
    entities
        .projectiles
        .iter()
        .map(|x| (x.id, x.entity.body.polygon.bounds_real.new))
        .chain(
            entities
                .torpedoes
                .iter()
                .map(|x| (x.id, x.entity.body.polygon.bounds_real.new)),
        )
        .for_each(|(id, bounds)| {
            if bounds.x + bounds.width < 0.0
                || bounds.x > COSMOS_WIDTH as f32
                || bounds.y + bounds.height < 0.0
                || bounds.y > COSMOS_HEIGHT as f32
            {
                dead.insert(id);
            }
        });
}
