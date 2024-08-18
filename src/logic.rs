use crate::{
    bus::Bus,
    commands::EntityCommands,
    components::{Motion, Regeneratable, Shape},
    constants::{COSMOS_HEIGHT, COSMOS_WIDTH},
    entities::{Entities, EntityIndex},
    forge::Forge,
    misc::{Node, NodeType, QuadTree},
};

use raylib::prelude::*;

const COSMIC_DRAG: Vector2 = Vector2::new(0.1, 0.1);
const COSMIC_DRAG_ROTATION: f32 = 0.003;

pub struct Logic {
    dead: Vec<usize>,
    quad_tree: QuadTree,
}

impl Logic {
    pub fn new() -> Self {
        Self {
            dead: Vec::new(),
            quad_tree: QuadTree::new(COSMOS_WIDTH, COSMOS_HEIGHT),
        }
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
        update_collision_detection(entities, &mut self.quad_tree);
        update_particles(entities, &mut self.dead);
        update_out_of_bounds(entities, &mut self.dead);
    }

    pub fn draw(&self, r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>) {
        self.quad_tree.draw(r);
    }
}

fn update_collision_detection(entities: &mut Entities, quad_tree: &mut QuadTree) {
    quad_tree.reset();

    (&entities.triships)
        .into_iter()
        .map(|x| x.id)
        .chain((&entities.projectiles).into_iter().map(|x| x.id))
        .for_each(|eid| {
            quad_tree.add(eid, &entities);
        });

    run(&quad_tree.root, entities);

    fn run(node: &Node, entities: &Entities) {
        match &node.node_type {
            NodeType::Leaf(eids) => {
                for i in 0..eids.len() {
                    let one = eids[i];
                    let v_one = get_vertexes(one, entities);

                    for j in i + 1..eids.len() {
                        let two = eids[j];
                        let v_two = get_vertexes(two, entities);

                        // TODO: collision check using v_one and v_two
                    }
                }
            }
            NodeType::Branch(nodes) => {
                for node in nodes {
                    run(&node, entities);
                }
            }
        }
    }

    fn get_vertexes(eidx: EntityIndex, entities: &Entities) -> &[Vector2] {
        match eidx {
            EntityIndex::Triship(idx) => &entities.triships[idx].entity.body.polygon.vertexes,
            EntityIndex::Projectile(idx) => &entities.projectiles[idx].entity.body.polygon.vertexes,
            EntityIndex::Exhaust(idx) => &entities.exhausts[idx].entity.body.polygon.vertexes,
        }
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
        .map(|x| {
            (
                &mut x.entity.body.generation as &mut dyn Regeneratable,
                &mut x.entity.body.polygon.bounds as &mut dyn Regeneratable,
            )
        })
        .chain((&mut entities.projectiles).into_iter().map(|x| {
            (
                &mut x.entity.body.generation as &mut dyn Regeneratable,
                &mut x.entity.body.polygon.bounds as &mut dyn Regeneratable,
            )
        }))
        .chain((&mut entities.exhausts).into_iter().map(|x| {
            (
                &mut x.entity.body.generation as &mut dyn Regeneratable,
                &mut x.entity.body.polygon.bounds as &mut dyn Regeneratable,
            )
        }))
        .for_each(|(body, bounds)| {
            body.regenerate();
            bounds.regenerate();
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
        .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion))
        .chain(
            (&mut entities.projectiles)
                .into_iter()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .chain(
            (&mut entities.exhausts)
                .into_iter()
                .map(|x| (&mut x.entity.body as &mut dyn Shape, &x.entity.motion)),
        )
        .for_each(|(shape, motion)| {
            shape.accelerate(motion.velocity);
            shape.rotate(motion.rotation_speed);
            shape.renew();
        });
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
