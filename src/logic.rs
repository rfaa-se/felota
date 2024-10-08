mod collisions;

use std::collections::BTreeSet;

use crate::{
    bus::Bus,
    commands::{Command, EntityCommands},
    components::{Centroidable, Generationable, Motion, Renewable, Shape, Targeting},
    constants::{COSMOS_HEIGHT, COSMOS_WIDTH, STARFIELD_HEIGHT, STARFIELD_WIDTH},
    entities::{Entities, EntityIndex},
    forge::Forge,
    messages::LogicMessage,
    quadtree::QuadTree,
    utils::generate_targeting_area,
};

use raylib::prelude::*;

use collisions::*;

// TODO: move to constants..?
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
        let dead = &mut self.dead;
        let commands = &mut self.commands;
        let quadtree = &mut self.quadtree;
        let collisions = &mut self.collisions;

        update_dead_removal(entities, dead);
        update_body_generation(entities);
        update_commands(entities, entity_cmds, commands, forge, quadtree, h);
        update_boost(entities);
        update_cooldowns(entities);
        update_motion(entities);
        update_body(entities);
        update_collision_detection(entities, quadtree, collisions);
        update_collision_reaction(entities, collisions, forge, h);
        update_targeting(entities);
        update_particles_exhaust_alpha(entities);
        update_particles_lifetime(entities, dead);
        update_particles_explosions(entities);
        update_particles_stars(entities);
        update_torpedo_timers(entities);
        update_targeting_rotation(entities, commands);
        update_commands_accelerate(entities, commands);
        update_out_of_bounds(entities, dead);
        update_dead_detection(entities, dead);
        update_dead_notify(entities, dead, bus);
    }

    pub fn draw(&self, r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>) {
        self.quadtree.draw(r);
    }
}

// TODO: move these functions into their own files? need to figure out structure

fn update_targeting_rotation(entities: &mut Entities, commands: &mut Vec<(usize, Command)>) {
    let targeter_target = entities
        .torpedoes
        .iter()
        .filter_map(|x| match x.entity.target {
            Some(target) => Some((
                (
                    x.id,
                    x.entity.body.state.new.rotation,
                    x.entity.body.state.new.shape.centroid(),
                    x.entity.timer_inactive == 0,
                ),
                target,
            )),
            None => None,
        })
        .collect::<Vec<_>>();

    targeter_target
        .iter()
        .for_each(|((eid, rotation, centroid, active), eid_target)| {
            let (eidx, velocity) = match entities.entity(*eid) {
                Some(eidx) => match eidx {
                    EntityIndex::Torpedo(idx) => {
                        let e = &entities.torpedoes[idx].entity;
                        (eidx, e.motion.velocity)
                    }
                    _ => panic!("wtf targeter 1 {:?}", eidx),
                },
                None => panic!("wtf target"),
            };

            let eidx_target = match entities.entity(*eid_target) {
                Some(eidx) => eidx,
                None => {
                    // target is dead, stop following
                    let target = match eidx {
                        EntityIndex::Torpedo(idx) => &mut entities.torpedoes[idx].entity.target,
                        _ => panic!("wtf target {:?}", eidx),
                    };

                    *target = None;

                    return;
                }
            };

            let centroid_target = match eidx_target {
                EntityIndex::Triship(idx) => {
                    let e = &entities.triships[idx].entity;
                    e.body.state.new.shape.centroid()
                }
                _ => panic!("wtf target centroid {:?}", eidx_target),
            };

            let direction = (centroid_target - *centroid).normalized();
            let cross = direction.x * rotation.y - direction.y * rotation.x;

            // without some kind of threshold, the torpedo will be very wobbly
            let threshold = 0.024;

            if cross < -threshold {
                commands.push((*eid, Command::RotateRight));
            } else if cross > threshold {
                commands.push((*eid, Command::RotateLeft));
            }

            if !active {
                return;
            }

            let mut cross_abs = cross.abs();
            let mut speed = velocity.length_sqr();

            let mut a = 0;
            if cross_abs > 0.4 {
                println!("{}", speed);
                while cross_abs > 0.2 && speed > 300.0 {
                    commands.push((*eid, Command::Decelerate));

                    cross_abs -= 0.1;
                    speed -= 100.0;
                    a += 1;
                    println!("{}", cross_abs);

                    if cross_abs < threshold * 1.5 {
                        if cross < -threshold {
                            commands.push((*eid, Command::RotateRight));
                        } else if cross > threshold {
                            commands.push((*eid, Command::RotateLeft));
                        }
                    }
                }

                println!("{} {}", speed, a);
            }
        });
}

fn update_targeting(entities: &mut Entities) {
    let targeter_target = entities
        .triships
        .iter()
        .map(|x| (x.id, x.entity.targeting.eid))
        .filter_map(|(tr_eid, te_eid)| match te_eid {
            Some(te_eid) => Some((tr_eid, te_eid)),
            None => None,
        })
        .collect::<Vec<_>>();

    targeter_target.iter().for_each(|(eid, eid_target)| {
        let eidx_target = entities.entity(*eid_target);
        let (eidx, centroid, rotation, angle) = match entities.entity(*eid) {
            Some(eidx) => match eidx {
                EntityIndex::Triship(idx) => {
                    let e = &entities.triships[idx].entity;
                    (
                        eidx,
                        e.body.state.new.shape.centroid(),
                        e.body.state.new.rotation,
                        e.targeting.angle,
                    )
                }
                _ => panic!("wtf targeter {:?}", eidx),
            },
            None => panic!("wtf target {}", eid),
        };

        let vert_target = match eidx_target {
            Some(eidx) => {
                // target has already been locked
                if target(entities, eidx).timer.current == 0 {
                    return;
                }

                match eidx {
                    EntityIndex::Triship(idx) => {
                        &entities.triships[idx].entity.body.polygon.vertexes.new
                    }
                    _ => panic!("wtf target {:?}", eidx),
                }
            }
            None => {
                // the targeted entity is dead
                reset(target(entities, eidx));
                return;
            }
        };

        let vert_area = generate_targeting_area(centroid, rotation, angle);
        let mut overlap = 0.0;
        let mut smallest = Vector2::zero();

        let axes_target = axes(&vert_target);
        if overlapping(
            &vert_target,
            &vert_area,
            &axes_target,
            &mut overlap,
            &mut smallest,
        ) {
            lock(target(entities, eidx));
            return;
        }

        let axes_area = axes(&vert_area);
        if overlapping(
            &vert_target,
            &vert_area,
            &axes_area,
            &mut overlap,
            &mut smallest,
        ) {
            lock(target(entities, eidx));
            return;
        }

        // target has been lost
        reset(target(entities, eidx));
    });

    fn reset(target: &mut Targeting) {
        target.eid = None;
        target.timer.current = target.timer.max;
    }

    fn lock(target: &mut Targeting) {
        if target.timer.current == 0 {
            return;
        }

        target.timer.current -= 1;
    }

    fn target(entities: &mut Entities, eidx: EntityIndex) -> &mut Targeting {
        match eidx {
            EntityIndex::Triship(idx) => &mut entities.triships[idx].entity.targeting,
            _ => panic!("wtf target {:?}", eidx),
        }
    }
}

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
            if x.entity.timer_inactive == 0 {
                Some((x.id, x.entity.life))
            } else {
                None
            }
        })
        .chain(entities.projectiles.iter().map(|x| (x.id, x.entity.life)))
        .filter_map(|(id, life)| if life > 0.0 { Some(id) } else { None })
        .for_each(|x| {
            commands.push((x, Command::Accelerate));
        });
}

fn update_cooldowns(entities: &mut Entities) {
    entities
        .triships
        .iter_mut()
        .map(|x| &mut x.entity.cooldown_torpedo.current)
        .filter(|x| **x != 0)
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

        if c.a >= r {
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
    quadtree: &QuadTree,
    h: &mut RaylibHandle,
) {
    for entity_cmd in entity_cmds {
        for cmd in entity_cmd.commands.iter() {
            cmd.execute(entities, entity_cmd.id, forge, quadtree, h);
        }
    }

    while let Some((id, cmd)) = entity_cmds_internal.pop() {
        cmd.execute(entities, id, forge, quadtree, h);
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
                .map(|x| (&mut x.entity.motion, true)),
        )
        .for_each(|(motion, apply_drag)| {
            if apply_drag {
                apply_cosmic_drag(motion);
            }

            check_speed_max(motion);
            check_rotation_speed_max(motion);
        });

    fn apply_cosmic_drag(motion: &mut Motion) {
        let direction = motion.velocity.normalized();
        let drag = direction * COSMIC_DRAG;

        motion.velocity -= drag;

        // if entity has suddenly switched direction after the drag, we should make a full stop
        if direction.dot(motion.velocity.normalized()) < 0.0 {
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
