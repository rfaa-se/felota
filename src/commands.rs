use crate::{
    components::Centroidable,
    entities::{Entities, Entity, EntityIndex},
    forge::Forge,
    quadtree::QuadTree,
    utils::generate_targeting_area,
};

use raylib::prelude::*;

pub struct EntityCommands {
    pub id: usize,
    pub commands: Box<[Command]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Command {
    Accelerate,
    Decelerate,
    RotateLeft,
    RotateRight,
    Projectile,
    Boost,
    Torpedo,
    Spawn(Spawn),
    TargetLock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Spawn {
    Triship(i32, i32),
}

enum Exhaust {
    Triship,
    Torpedo,
}

impl Command {
    const ACCELERATE: u8 = 1;
    const DECELERATE: u8 = 2;
    const ROTATE_LEFT: u8 = 3;
    const ROTATE_RIGHT: u8 = 4;
    const PROJECTILE: u8 = 5;
    const BOOST: u8 = 6;
    const TORPEDO: u8 = 7;
    const SPAWN: u8 = 8;
    const TARGET_LOCK: u8 = 9;

    pub fn execute(
        &self,
        entities: &mut Entities,
        eid: usize,
        forge: &Forge,
        quadtree: &QuadTree,
        h: &mut RaylibHandle,
    ) {
        let Some(eidx) = entities.entity(eid) else {
            return;
        };

        match self {
            Command::Accelerate => handle_accelerate(entities, eidx, forge, h),
            Command::Decelerate => handle_decelerate(entities, eidx, forge, h),
            Command::RotateLeft => handle_rotate_left(entities, eidx, forge, h),
            Command::RotateRight => handle_rotate_right(entities, eidx, forge, h),
            Command::Projectile => handle_projectile(entities, eidx, eid, forge),
            Command::Boost => handle_boost(entities, eidx),
            Command::Torpedo => handle_torpedo(entities, eidx, eid, forge),
            Command::Spawn(spawn) => handle_spawn(entities, forge, spawn),
            Command::TargetLock => handle_target_lock(entities, eidx, quadtree),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let Some((ctype, data)) = bytes.split_first() else {
            panic!("wtf cmd");
        };

        match *ctype {
            Self::ACCELERATE => Command::Accelerate,
            Self::DECELERATE => Command::Decelerate,
            Self::ROTATE_LEFT => Command::RotateLeft,
            Self::ROTATE_RIGHT => Command::RotateRight,
            Self::PROJECTILE => Command::Projectile,
            Self::BOOST => Command::Boost,
            Self::TORPEDO => Command::Torpedo,
            Self::SPAWN => Command::Spawn(Spawn::from_bytes(data)),
            Self::TARGET_LOCK => Command::TargetLock,
            _ => panic!("wtf ctype {}", ctype),
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        bytes.push(self.len());

        match self {
            Command::Accelerate => bytes.push(Self::ACCELERATE),
            Command::Decelerate => bytes.push(Self::DECELERATE),
            Command::RotateLeft => bytes.push(Self::ROTATE_LEFT),
            Command::RotateRight => bytes.push(Self::ROTATE_RIGHT),
            Command::Projectile => bytes.push(Self::PROJECTILE),
            Command::Boost => bytes.push(Self::BOOST),
            Command::Torpedo => bytes.push(Self::TORPEDO),
            Command::Spawn(spawn) => {
                bytes.push(Self::SPAWN);
                bytes.extend_from_slice(&spawn.to_bytes().into_vec());
            }
            Command::TargetLock => bytes.push(Self::TARGET_LOCK),
        }

        bytes.into_boxed_slice()
    }

    pub fn len(&self) -> u8 {
        // type identifier + potential data length in command
        1 + match self {
            Command::Spawn(spawn) => spawn.len(),
            _ => 0,
        }
    }
}

impl Spawn {
    const TRISHIP: u8 = 1;

    pub fn from_bytes(bytes: &[u8]) -> Self {
        // length will be the first byte, don't care about it in here
        let Some((stype, data)) = bytes[1..].split_first() else {
            panic!("wtf spawn");
        };

        match *stype {
            Self::TRISHIP => {
                let (x, y) = data.split_at(4);
                let x = i32::from_be_bytes(x.try_into().expect("wtf spawn x"));
                let y = i32::from_be_bytes(y.try_into().expect("wtf spawn y"));

                Spawn::Triship(x, y)
            }
            _ => panic!("wtf stype {}", stype),
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        bytes.push(self.len());

        match self {
            Spawn::Triship(x, y) => {
                bytes.push(Self::TRISHIP);
                bytes.extend_from_slice(&x.to_be_bytes());
                bytes.extend_from_slice(&y.to_be_bytes());
            }
        }

        bytes.into_boxed_slice()
    }

    pub fn len(&self) -> u8 {
        1 + 1 // length itself + type identifier + potential data length in spawn
            + match self {
                Spawn::Triship(_, _) => 4 + 4,
            }
    }
}

fn handle_spawn(entities: &mut Entities, forge: &Forge, spawn: &Spawn) {
    let entity = match spawn {
        Spawn::Triship(x, y) => Entity::Triship(forge.triship(Vector2::new(*x as f32, *y as f32))),
    };

    entities.add(entity);
}

fn handle_accelerate(
    entities: &mut Entities,
    eidx: EntityIndex,
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    let (rotation, motion) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (e.body.state.new.rotation, &mut e.motion)
        }
        EntityIndex::Torpedo(idx) => {
            let e = &mut entities.torpedoes[idx].entity;
            (e.body.state.new.rotation, &mut e.motion)
        }
        EntityIndex::Projectile(idx) => {
            let e = &mut entities.projectiles[idx].entity;
            (e.body.state.new.rotation, &mut e.motion)
        }
        _ => panic!("wtf accelerate {:?}", eidx),
    };

    motion.velocity += rotation * motion.acceleration;

    // spawn exhaust particles
    let initial_velocity = motion.velocity;
    // e-type o7
    let (position, etype) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &entities.triships[idx].entity;
            let v = &e.body.polygon.vertexes.new;

            (
                Vector2::new((v[0].x + v[2].x) / 2.0, (v[0].y + v[2].y) / 2.0),
                Exhaust::Triship,
            )
        }
        // no exhaust if torpedo is still inactive
        EntityIndex::Torpedo(idx) if entities.torpedoes[idx].entity.timer_inactive == 0 => {
            let e = &entities.torpedoes[idx].entity;
            let v = &e.body.polygon.vertexes.new;

            (
                Vector2::new((v[0].x + v[3].x) / 2.0, (v[0].y + v[3].y) / 2.0),
                Exhaust::Torpedo,
            )
        }
        _ => return,
    };

    // rotate 180 degrees, we want the exhaust to be pointed away from the rotation of the entity
    let exhaust_rotation = Vector2 {
        x: rotation.x * -1.0,
        y: rotation.y * -1.0,
    };

    let exhausts = match etype {
        Exhaust::Triship => {
            forge.exhaust_afterburner(position, exhaust_rotation, initial_velocity, h)
        }
        Exhaust::Torpedo => forge.exhaust_torpedo(position, exhaust_rotation, initial_velocity, h),
    };

    for exhaust in exhausts {
        entities.add(Entity::Exhaust(exhaust));
    }
}

fn handle_decelerate(
    entities: &mut Entities,
    eidx: EntityIndex,
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    let (rotation, motion) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (e.body.state.new.rotation, &mut e.motion)
        }
        EntityIndex::Torpedo(idx) => {
            let e = &mut entities.torpedoes[idx].entity;
            (e.body.state.new.rotation, &mut e.motion)
        }
        _ => panic!("wtf decelerate {:?}", eidx),
    };

    motion.velocity -= rotation * (motion.acceleration / 4.0);

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let triship = &entities.triships[idx].entity;
        let initial_velocity = triship.motion.velocity;
        let vertexes = &triship.body.polygon.vertexes.new;

        // calculate the placement position of the left thruster
        let position_left = Vector2::new(
            vertexes[2].x * 0.2 + vertexes[1].x * 0.8,
            vertexes[2].y * 0.2 + vertexes[1].y * 0.8,
        );

        // calculate the placement position of the right thruster
        let position_right = Vector2::new(
            vertexes[0].x * 0.2 + vertexes[1].x * 0.8,
            vertexes[0].y * 0.2 + vertexes[1].y * 0.8,
        );

        for exhaust in
            forge.exhaust_thruster_bow(position_left, position_right, rotation, initial_velocity, h)
        {
            entities.add(Entity::Exhaust(exhaust));
        }
    }
}

fn handle_rotate_left(
    entities: &mut Entities,
    eidx: EntityIndex,
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    let (motion, old_rotation) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (&mut e.motion, e.body.state.old.rotation)
        }
        EntityIndex::Torpedo(idx) => {
            let e = &mut entities.torpedoes[idx].entity;
            (&mut e.motion, e.body.state.old.rotation)
        }
        _ => panic!("wtf rotate left {:?}", eidx),
    };

    motion.rotation_speed -= motion.rotation_acceleration;

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let triship = &entities.triships[idx].entity;
        let initial_velocity = triship.motion.velocity;
        let vertexes = &triship.body.polygon.vertexes.new;

        // calculate the placement position of the thruster
        let position = Vector2::new(
            vertexes[2].x * 0.2 + vertexes[1].x * 0.8,
            vertexes[2].y * 0.2 + vertexes[1].y * 0.8,
        );

        // rotate 270 degrees, we want the exhaust to be pointed to the right of the entity
        let exhaust_rotation = Vector2 {
            x: old_rotation.y * -1.0,
            y: old_rotation.x,
        };

        for exhaust in forge.exhaust_thruster(position, exhaust_rotation, initial_velocity, h) {
            entities.add(Entity::Exhaust(exhaust));
        }
    }
}

fn handle_rotate_right(
    entities: &mut Entities,
    eidx: EntityIndex,
    forge: &Forge,
    h: &mut RaylibHandle,
) {
    let (motion, old_rotation) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (&mut e.motion, e.body.state.old.rotation)
        }
        EntityIndex::Torpedo(idx) => {
            let e = &mut entities.torpedoes[idx].entity;
            (&mut e.motion, e.body.state.old.rotation)
        }
        _ => panic!("wtf rotate right {:?}", eidx),
    };

    motion.rotation_speed += motion.rotation_acceleration;

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let triship = &entities.triships[idx].entity;
        let initial_velocity = triship.motion.velocity;
        let vertexes = &triship.body.polygon.vertexes.new;

        // calculate the placement position of the right thruster
        let position = Vector2::new(
            vertexes[0].x * 0.2 + vertexes[1].x * 0.8,
            vertexes[0].y * 0.2 + vertexes[1].y * 0.8,
        );

        // rotate 90 degrees, we want the exhaust to be pointed to the right of the entity
        let exhaust_rotation = Vector2 {
            x: old_rotation.y,
            y: old_rotation.x * -1.0,
        };

        for exhaust in forge.exhaust_thruster(position, exhaust_rotation, initial_velocity, h) {
            entities.add(Entity::Exhaust(exhaust));
        }
    }
}

fn handle_projectile(entities: &mut Entities, eidx: EntityIndex, id: usize, forge: &Forge) {
    let (body, velocity) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &entities.triships[idx].entity;
            (&e.body, e.motion.velocity)
        }
        _ => panic!("wtf projectile {:?}", eidx),
    };

    let rotation = body.state.new.rotation;
    let position = body.polygon.vertexes.new[1];
    let projectile = forge.projectile(position, rotation, velocity, id);

    entities.add(Entity::Projectile(projectile));
}

fn handle_boost(entities: &mut Entities, eidx: EntityIndex) {
    let (motion, boost) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (&mut e.motion, &mut e.boost)
        }
        _ => panic!("wtf boost {:?}", eidx),
    };

    if boost.active {
        return;
    }

    boost.active = true;
    boost.lifetime.current = boost.lifetime.max;
    boost.cooldown.current = boost.cooldown.max;
    boost.speed_max_old = motion.speed_max;
    boost.acceleration_old = motion.acceleration;

    motion.speed_max = boost.speed_max;
    motion.acceleration = boost.acceleration;
}

fn handle_torpedo(entities: &mut Entities, eidx: EntityIndex, id: usize, forge: &Forge) {
    let (body, velocity, cooldown, target) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (
                &e.body,
                e.motion.velocity,
                &mut e.cooldown_torpedo,
                &e.targeting,
            )
        }
        _ => panic!("wtf torpedo {:?}", eidx),
    };

    if cooldown.current != 0 {
        return;
    }

    cooldown.current = cooldown.max;

    // only use target if it's been locked
    let target = if target.timer.current == 0 {
        target.eid
    } else {
        None
    };

    let rotation = body.state.new.rotation;
    let vertexes = &body.polygon.vertexes.new;
    let position = Vector2::new(
        vertexes[0].x * 0.4 + vertexes[1].x * 0.6,
        vertexes[0].y * 0.4 + vertexes[1].y * 0.6,
    );

    let torpedo = forge.torpedo(position, rotation, velocity, id, target);

    entities.add(Entity::Torpedo(torpedo));
}

fn handle_target_lock(entities: &mut Entities, eidx: EntityIndex, quadtree: &QuadTree) {
    let (centroid, eid_target) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (e.body.state.new.shape.centroid(), e.targeting.eid)
        }
        _ => panic!("wtf target lock {:?}", eidx),
    };

    let area = generate_targeting_area(centroid);

    let mut targets = quadtree
        .get(&area, entities)
        .iter()
        .filter_map(|x| {
            // don't target self
            if *x == eidx {
                return None;
            }

            let c = match *x {
                EntityIndex::Triship(idx) => entities.triships[idx]
                    .entity
                    .body
                    .state
                    .new
                    .shape
                    .centroid(),
                _ => return None,
            };

            let cx = c.x - centroid.x;
            let cy = c.y - centroid.y;

            let dist_sqr = (cx * cx) + (cy * cy);

            Some((*x, dist_sqr))
        })
        .collect::<Vec<_>>();

    targets.sort_unstable_by(|(_, dist_a), (_, dist_b)| dist_a.total_cmp(dist_b));

    let idx_current = match eid_target {
        Some(eid) => targets.iter().position(|(eidx, _)| match eidx {
            EntityIndex::Triship(idx) => entities.triships[*idx].id == eid,
            _ => false,
        }),
        None => None,
    };

    let eidx_target = match idx_current {
        Some(idx) if idx + 1 < targets.len() => {
            let (eidx, _) = targets[idx + 1];
            Some(eidx)
        }
        _ => {
            if targets.len() == 0 || idx_current.is_some() {
                None
            } else {
                let (eidx, _) = targets[0];
                Some(eidx)
            }
        }
    };

    let eid_target = match eidx_target {
        Some(eidx) => match eidx {
            EntityIndex::Triship(idx) => Some(entities.triships[idx].id),
            _ => None,
        },
        None => None,
    };

    let targeting = match eidx {
        EntityIndex::Triship(idx) => &mut entities.triships[idx].entity.targeting,
        _ => panic!("wtf targeting {:?}", eidx),
    };

    targeting.eid = eid_target;
    targeting.timer.current = targeting.timer.max;
}
