use std::collections::BTreeSet;

use raylib::prelude::*;

use crate::{
    components::{Boundable, Generation, Shape},
    entities::{Entities, Entity, EntityIndex},
    forge::Forge,
    misc::{Node, NodeType, QuadTree},
};

pub fn update_collision_detection(
    entities: &mut Entities,
    quadtree: &mut QuadTree,
    dead: &mut BTreeSet<usize>,
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    quadtree.reset();

    entities
        .triships
        .iter()
        .map(|x| x.id)
        .chain(entities.projectiles.iter().map(|x| x.id))
        .for_each(|eid| {
            quadtree.add(eid, &entities);
        });

    run(&quadtree.root, entities, dead, forge, h);

    fn run(
        node: &Node,
        entities: &mut Entities,
        dead: &mut BTreeSet<usize>,
        forge: &Forge,
        h: &mut RaylibHandle,
    ) {
        match &node.node_type {
            NodeType::Leaf(ents) => {
                for i in 0..ents.len() {
                    let (eidx1, eid1) = ents[i];
                    let bounds1 = bounds(eidx1, entities);

                    for j in i + 1..ents.len() {
                        let (eidx2, eid2) = ents[j];

                        match (eidx1, eidx2) {
                            // let's not shoot ourselves
                            // TODO: maybe this should be possible?
                            (EntityIndex::Triship(idx1), EntityIndex::Projectile(idx2))
                            | (EntityIndex::Projectile(idx2), EntityIndex::Triship(idx1))
                                if entities.projectiles[idx2].entity.owner_id
                                    == entities.triships[idx1].id =>
                            {
                                continue;
                            }
                            // let's not shoot projectiles with projectiles
                            (EntityIndex::Projectile(_), EntityIndex::Projectile(_)) => {
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
                                (EntityIndex::Triship(_), EntityIndex::Triship(_)) => {
                                    // TODO: boom?
                                }
                                (EntityIndex::Triship(idx_t), EntityIndex::Projectile(idx_p)) => {
                                    handle_repositioning(
                                        eidx2,
                                        dir2 * -1.0 * speed_max2 + dir2 * speed_cur2,
                                        entities,
                                    );

                                    dead.insert(eid2);

                                    handle_triship_projectile(idx_t, idx_p, entities, forge, h);
                                }
                                (EntityIndex::Projectile(idx_p), EntityIndex::Triship(idx_t)) => {
                                    handle_repositioning(
                                        eidx1,
                                        dir1 * -1.0 * speed_max1 + dir1 * speed_cur1,
                                        entities,
                                    );

                                    dead.insert(eid1);

                                    handle_triship_projectile(idx_t, idx_p, entities, forge, h);
                                }
                                _ => (),
                            }

                            fn handle_repositioning(
                                eidx: EntityIndex,
                                vel: Vector2,
                                entities: &mut Entities,
                            ) {
                                let s = shape(eidx, entities);
                                s.accelerate(vel);
                                s.renew();
                            }

                            fn handle_triship_projectile(
                                idx_t: usize,
                                idx_p: usize,
                                entities: &mut Entities,
                                forge: &Forge,
                                h: &mut RaylibHandle,
                            ) {
                                let t = &mut entities.triships[idx_t].entity;
                                let p = &entities.projectiles[idx_p].entity;

                                t.life -= p.damage;

                                // spawn some explosions!
                                for explosion in
                                    forge.explosion_projectile(p.body.polygon.vertexes.new[1], h)
                                {
                                    entities.add(Entity::Explosion(explosion));
                                }
                            }

                            break;
                        }
                    }
                }
            }
            NodeType::Branch(nodes) => {
                for node in nodes {
                    run(&node, entities, dead, forge, h);
                }
            }
        }
    }

    fn vertexes(eidx: EntityIndex, entities: &Entities) -> &Generation<Vec<Vector2>> {
        match eidx {
            EntityIndex::Triship(idx) => &entities.triships[idx].entity.body.polygon.vertexes,
            EntityIndex::Projectile(idx) => &entities.projectiles[idx].entity.body.polygon.vertexes,
            _ => panic!("vertexes {:?}", eidx),
        }
    }

    fn bounds(eidx: EntityIndex, entities: &Entities) -> Rectangle {
        match eidx {
            EntityIndex::Triship(idx) => &entities.triships[idx].entity.body.polygon,
            EntityIndex::Projectile(idx) => &entities.projectiles[idx].entity.body.polygon,
            _ => panic!("bounds {:?}", eidx),
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
            _ => panic!("shape {:?}", eidx),
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
