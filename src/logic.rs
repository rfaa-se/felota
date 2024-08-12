use crate::{
    bus::Bus,
    commands::EntityCommands,
    components::{Acceleratable, Motion, Renewable},
    constants::{COSMOS_HEIGHT, COSMOS_WIDTH},
    entities::Entities,
    forge::Forge,
};

use raylib::prelude::*;

const COSMIC_DRAG: Vector2 = Vector2::new(0.1, 0.1);
const COSMIC_DRAG_ROTATION: f32 = 0.003;

pub struct Logic {
    dead: Vec<usize>,
}

impl Logic {
    pub fn new() -> Self {
        Self { dead: Vec::new() }
    }

    pub fn update(
        &mut self,
        _bus: &mut Bus,
        entities: &mut Entities,
        entity_cmds: &[EntityCommands],
        forge: &Forge,
        h: &mut RaylibHandle,
    ) {
        update_dead(entities, &mut self.dead);
        update_body_generation(entities);
        update_commands(entities, entity_cmds, forge, h);
        update_motion(entities);
        update_body(entities);
        update_particles(entities, &mut self.dead);
        update_out_of_bounds(entities, &mut self.dead);
    }
}

fn update_particles(entities: &mut Entities, dead: &mut Vec<usize>) {
    (&mut entities.exhausts).into_iter().for_each(|x| {
        if x.entity.lifetime == 0 {
            dead.push(x.id);
        } else {
            x.entity.lifetime -= 1;
        }
    });
}

fn update_dead(entities: &mut Entities, dead: &mut Vec<usize>) {
    while let Some(d) = dead.pop() {
        entities.kill(d);
    }
}

fn update_body_generation(entities: &mut Entities) {
    (&mut entities.triships)
        .into_iter()
        .map(|x| &mut x.entity.body.generation as &mut dyn Renewable)
        .chain(
            (&mut entities.projectiles)
                .into_iter()
                .map(|x| &mut x.entity.body.generation as &mut dyn Renewable),
        )
        .chain(
            (&mut entities.exhausts)
                .into_iter()
                .map(|x| &mut x.entity.body.generation as &mut dyn Renewable),
        )
        .for_each(|x| {
            x.renew();
        })
}

fn update_commands(
    entities: &mut Entities,
    entity_cmds: &[EntityCommands],
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    for entity_cmd in entity_cmds {
        for cmd in entity_cmd.commands.iter() {
            cmd.execute(entities, entity_cmd.id, forge, h);
        }
    }
}

fn update_motion(entities: &mut Entities) {
    (&mut entities.triships)
        .into_iter()
        .map(|x| (&mut x.entity.motion, true))
        .chain(
            (&mut entities.projectiles)
                .into_iter()
                .map(|x| (&mut x.entity.motion, false)),
        )
        .chain(
            (&mut entities.exhausts)
                .into_iter()
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
        if motion.velocity.length() > motion.speed_max {
            motion.velocity = motion.velocity.normalized() * motion.speed_max;
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
    (&mut entities.triships)
        .into_iter()
        .map(|x| {
            (
                &mut x.entity.body.generation.new.shape as &mut dyn Acceleratable,
                &x.entity.motion,
                &mut x.entity.body.generation.new.rotation,
            )
        })
        .chain((&mut entities.projectiles).into_iter().map(|x| {
            (
                &mut x.entity.body.generation.new.shape as &mut dyn Acceleratable,
                &x.entity.motion,
                &mut x.entity.body.generation.new.rotation,
            )
        }))
        .chain((&mut entities.exhausts).into_iter().map(|x| {
            (
                &mut x.entity.body.generation.new.shape as &mut dyn Acceleratable,
                &x.entity.motion,
                &mut x.entity.body.generation.new.rotation,
            )
        }))
        .for_each(|(shape, motion, rotation)| {
            shape.accelerate(motion.velocity);
            update_rotation(rotation, motion);
        });

    fn update_rotation(rotation: &mut Vector2, motion: &Motion) {
        let radians = rotation.y.atan2(rotation.x) + motion.rotation_speed;
        (rotation.y, rotation.x) = radians.sin_cos();
    }
}

fn update_out_of_bounds(entities: &mut Entities, dead: &mut Vec<usize>) {
    (&mut entities.projectiles)
        .into_iter()
        .map(|x| (x.id, &mut x.entity.body))
        .for_each(|(id, body)| {
            let s = body.generation.new.shape;
            let max = s.width.max(s.height);

            if s.x - max < 0.0
                || s.x > COSMOS_WIDTH as f32
                || s.y - max < 0.0
                || s.y > COSMOS_HEIGHT as f32
            {
                dead.push(id);
            }
        });
}
