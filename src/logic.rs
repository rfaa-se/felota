use crate::{
    bus::Bus,
    commands::EntityCommands,
    components::{Boundable, Generation, Generationable, Motion, Shape},
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
        update_collision_detection(entities, &mut self.quad_tree, &mut self.dead);
        update_particles(entities, &mut self.dead);
        update_out_of_bounds(entities, &mut self.dead);
    }

    pub fn draw(&self, r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>) {
        self.quad_tree.draw(r);
    }
}

fn update_collision_detection(
    entities: &mut Entities,
    quad_tree: &mut QuadTree,
    dead: &mut Vec<usize>,
) {
    quad_tree.reset();

    (&entities.triships)
        .into_iter()
        .map(|x| x.id)
        .chain((&entities.projectiles).into_iter().map(|x| x.id))
        .for_each(|eid| {
            quad_tree.add(eid, &entities);
        });

    run(&quad_tree.root, entities, dead);

    fn run(node: &Node, entities: &mut Entities, dead: &mut Vec<usize>) {
        match &node.node_type {
            NodeType::Leaf(ents) => {
                for i in 0..ents.len() {
                    let (eidx1, eid1) = ents[i];
                    let bounds1 = bounds(eidx1, entities);

                    for j in i + 1..ents.len() {
                        let (eidx2, eid2) = ents[j];

                        // let's not shoot ourselves
                        // TODO: maybe this should be possible?
                        match (eidx1, eidx2) {
                            (EntityIndex::Triship(idx1), EntityIndex::Projectile(idx2))
                            | (EntityIndex::Projectile(idx2), EntityIndex::Triship(idx1))
                                if entities.projectiles[idx2].entity.owner_id
                                    == entities.triships[idx1].id =>
                            {
                                continue;
                            }
                            _ => (),
                        }

                        let bounds2 = bounds(eidx2, entities);

                        // perform initial collision check using the meld bounds,
                        // if there's a hit, we perform a more thorough collision check
                        // using the SAT (separating axis theorem) while incrementally moving
                        // the entities from the old to the new location at most 1.0 distance at a time
                        // TODO: there's probably a more elegant way to solve this, but me grug brain
                        if !bounds1.check_collision_recs(&bounds2) {
                            continue;
                        }

                        // TODO: probably better to calculate all the stuff for the first entity outside this loop
                        let vert1 = vertexes(eidx1, entities);
                        let vert2 = vertexes(eidx2, entities);

                        let vel1 = vert1.new[0] - vert1.old[0];
                        let vel2 = vert2.new[0] - vert2.old[0];

                        let dir1 = vel1.normalized();
                        let dir2 = vel2.normalized();

                        let speed_max1 = vel1.length();
                        let speed_max2 = vel2.length();

                        // we want to keep the increments at 1.0 or less
                        let mut speed_incr1 = speed_max1;
                        let mut speed_incr2 = speed_max2;
                        while speed_incr1 > 1.0 || speed_incr2 > 1.0 {
                            speed_incr1 /= 2.0;
                            speed_incr2 /= 2.0;
                        }

                        let vert_old1 = &vert1.old;
                        let vert_old2 = &vert2.old;

                        let mut speed_cur1 = 0.0;
                        let mut speed_cur2 = 0.0;

                        loop {
                            speed_cur1 += speed_incr1;
                            speed_cur2 += speed_incr2;

                            // no movement, should probably already have collided
                            if speed_cur1 == 0.0 && speed_cur2 == 0.0 {
                                break;
                            }

                            // no collision!
                            if speed_cur1 > speed_max1 || speed_cur2 > speed_max2 {
                                break;
                            }

                            // keep moving the vertexes incrementally until we find the first collision
                            let vert_cur1 = vert_old1
                                .iter()
                                .map(|x| *x + dir1 * speed_cur1)
                                .collect::<Vec<_>>();

                            let vert_cur2 = vert_old2
                                .iter()
                                .map(|x| *x + dir2 * speed_cur2)
                                .collect::<Vec<_>>();

                            let mut overlap = f32::MAX;
                            let mut smallest = Vector2::zero();

                            let axes1 = axes(&vert_cur1);
                            if !overlapping(
                                &vert_cur1,
                                &vert_cur2,
                                &axes1,
                                &mut overlap,
                                &mut smallest,
                            ) {
                                continue;
                            }

                            let axes2 = axes(&vert_cur2);
                            if !overlapping(
                                &vert_cur1,
                                &vert_cur2,
                                &axes2,
                                &mut overlap,
                                &mut smallest,
                            ) {
                                continue;
                            }

                            match (eidx1, eidx2) {
                                (EntityIndex::Triship(_), EntityIndex::Projectile(_)) => {
                                    fix(
                                        eidx2,
                                        dir2 * -1.0 * speed_max2 + dir2 * speed_cur2,
                                        entities,
                                    );

                                    dead.push(eid2);
                                }
                                (EntityIndex::Projectile(_), EntityIndex::Triship(_)) => {
                                    fix(
                                        eidx1,
                                        dir1 * -1.0 * speed_max1 + dir1 * speed_cur1,
                                        entities,
                                    );

                                    dead.push(eid1);
                                }
                                (EntityIndex::Projectile(_), EntityIndex::Projectile(_)) => {
                                    fix(
                                        eidx1,
                                        dir1 * -1.0 * speed_max1 + dir1 * speed_cur1,
                                        entities,
                                    );
                                    fix(
                                        eidx2,
                                        dir2 * -1.0 * speed_max2 + dir2 * speed_cur2,
                                        entities,
                                    );

                                    dead.push(eid1);
                                    dead.push(eid2);
                                }
                                _ => (),
                            }

                            fn fix(eidx: EntityIndex, vel: Vector2, entities: &mut Entities) {
                                let s = shape(eidx, entities);
                                s.accelerate(vel);
                                s.renew();
                            }

                            break;
                        }
                    }
                }
            }
            NodeType::Branch(nodes) => {
                for node in nodes {
                    run(&node, entities, dead);
                }
            }
        }
    }

    fn vertexes(eidx: EntityIndex, entities: &Entities) -> &Generation<Vec<Vector2>> {
        match eidx {
            EntityIndex::Triship(idx) => &entities.triships[idx].entity.body.polygon.vertexes,
            EntityIndex::Projectile(idx) => &entities.projectiles[idx].entity.body.polygon.vertexes,
            EntityIndex::Exhaust(idx) => &entities.exhausts[idx].entity.body.polygon.vertexes,
        }
    }

    fn bounds(eidx: EntityIndex, entities: &Entities) -> Rectangle {
        match eidx {
            EntityIndex::Triship(idx) => &entities.triships[idx].entity.body.polygon,
            EntityIndex::Projectile(idx) => &entities.projectiles[idx].entity.body.polygon,
            EntityIndex::Exhaust(idx) => &entities.exhausts[idx].entity.body.polygon,
        }
        .bounds_meld
        .bounds()
    }

    fn shape(eidx: EntityIndex, entities: &mut Entities) -> &mut dyn Shape {
        match eidx {
            EntityIndex::Triship(idx) => &mut entities.triships[idx].entity.body as &mut dyn Shape,
            EntityIndex::Projectile(idx) => {
                &mut entities.projectiles[idx].entity.body as &mut dyn Shape
            }
            EntityIndex::Exhaust(idx) => &mut entities.exhausts[idx].entity.body as &mut dyn Shape,
        }
    }

    fn axes(vertexes: &[Vector2]) -> Vec<Vector2> {
        let mut axes = Vec::new();

        for i in 0..vertexes.len() {
            let v1 = vertexes[i];
            let v2 = vertexes[if i + 1 == vertexes.len() { 0 } else { i + 1 }];
            let edge = v1 - v2;
            // if we don't want the mtv(minimal translation vector), we don't need to normalize
            let norm = Vector2::new(-edge.y, edge.x).normalized();

            axes.push(norm);
        }

        axes
    }

    fn project(vertexes: &[Vector2], axis: Vector2) -> Vector2 {
        let mut min = axis.dot(vertexes[0]);
        let mut max = min;

        for i in 1..vertexes.len() {
            let p = axis.dot(vertexes[i]);

            if p < min {
                min = p;
            } else if p > max {
                max = p;
            }
        }

        Vector2::new(min, max)
    }

    fn overlapping(
        v_one: &[Vector2],
        v_two: &[Vector2],
        axes: &[Vector2],
        overlap: &mut f32,
        smallest: &mut Vector2,
    ) -> bool {
        for axis in axes {
            let p_one = project(v_one, *axis);
            let p_two = project(v_two, *axis);

            if !(p_one.y > p_two.x || p_one.x > p_two.y) {
                return false;
            }

            let mut o = p_one.y.min(p_two.y) - p_one.x.max(p_two.x);

            if contains(p_one, p_two) || contains(p_two, p_one) {
                let min = (p_one.x - p_two.x).abs();
                let max = (p_one.y - p_two.y).abs();

                if min < max {
                    o += min;
                } else {
                    o += max;
                }
            }

            // if we don't want the mtv(minimal translation vector), we can remove overlap and smallest
            if o < *overlap {
                *overlap = o;
                *smallest = *axis;
            }
        }

        true
    }

    fn contains(p_one: Vector2, p_two: Vector2) -> bool {
        p_one.x <= p_two.x && p_one.y >= p_two.y
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
        .map(|x| (&mut x.entity.body as &mut dyn Generationable))
        .chain(
            (&mut entities.projectiles)
                .into_iter()
                .map(|x| (&mut x.entity.body as &mut dyn Generationable)),
        )
        .chain(
            (&mut entities.exhausts)
                .into_iter()
                .map(|x| (&mut x.entity.body as &mut dyn Generationable)),
        )
        .for_each(|body| {
            body.generation();
        });
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
        .map(|x| (x.id, x.entity.body.polygon.bounds_real.new))
        .for_each(|(id, bounds)| {
            if bounds.x + bounds.width < 0.0
                || bounds.x > COSMOS_WIDTH as f32
                || bounds.y + bounds.height < 0.0
                || bounds.y > COSMOS_HEIGHT as f32
            {
                dead.push(id);
            }
        });
}
