use crate::{
    entities::{Entities, Entity, EntityIndex},
    forge::Forge,
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
}

const ACCELERATE: u8 = 1;
const DECELERATE: u8 = 2;
const ROTATE_LEFT: u8 = 3;
const ROTATE_RIGHT: u8 = 4;
const PROJECTILE: u8 = 5;
const BOOST: u8 = 6;

impl Command {
    pub fn execute(
        &self,
        entities: &mut Entities,
        eid: usize,
        forge: &Forge,
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
            Command::Boost => handle_boost(entities, eidx, eid, forge),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let Some((ctype, _data)) = bytes.split_first() else {
            panic!("wtf cmd");
        };

        match *ctype {
            ACCELERATE => Command::Accelerate,
            DECELERATE => Command::Decelerate,
            ROTATE_LEFT => Command::RotateLeft,
            ROTATE_RIGHT => Command::RotateRight,
            PROJECTILE => Command::Projectile,
            BOOST => Command::Boost,
            _ => panic!("wtf ctype {}", ctype),
        }
    }

    pub fn to_bytes(&self) -> Box<[u8]> {
        let mut bytes = Vec::new();

        // for now all commands are 1 in length
        let len = match self {
            _ => 1,
        };

        bytes.push(len);

        match self {
            Command::Accelerate => bytes.push(ACCELERATE),
            Command::Decelerate => bytes.push(DECELERATE),
            Command::RotateLeft => bytes.push(ROTATE_LEFT),
            Command::RotateRight => bytes.push(ROTATE_RIGHT),
            Command::Projectile => bytes.push(PROJECTILE),
            Command::Boost => bytes.push(BOOST),
        }

        bytes.into_boxed_slice()
    }
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
        _ => return,
    };

    motion.velocity += rotation * motion.acceleration;

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let initial_velocity = motion.velocity;
        let triship = &entities.triships[idx].entity;
        let vertexes = &triship.body.polygon.vertexes.new;

        // calculate the placement position of the afterburner
        let position = Vector2::new(
            (vertexes[0].x + vertexes[2].x) / 2.0,
            (vertexes[0].y + vertexes[2].y) / 2.0,
        );

        // rotate 180 degrees, we want the exhaust to be pointed away from the rotation of the entity
        let exhaust_rotation = Vector2 {
            x: rotation.x * -1.0,
            y: rotation.y * -1.0,
        };

        for exhaust in forge.exhaust_afterburner(position, exhaust_rotation, initial_velocity, h) {
            entities.add(Entity::Exhaust(exhaust));
        }
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
        _ => return,
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
        _ => return,
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

        for exhaust in forge.exhaust_thruster_side(position, exhaust_rotation, initial_velocity, h)
        {
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
        _ => return,
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

        for exhaust in forge.exhaust_thruster_side(position, exhaust_rotation, initial_velocity, h)
        {
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
        _ => return,
    };

    let rotation = body.state.new.rotation;
    let position = body.polygon.vertexes.new[1];
    let projectile = forge.projectile(position, rotation, velocity, id);

    entities.add(Entity::Projectile(projectile));
}

fn handle_boost(entities: &mut Entities, eidx: EntityIndex, _id: usize, _forge: &Forge) {
    let (motion, boost) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (&mut e.motion, &mut e.boost)
        }
        _ => return,
    };

    if boost.active {
        return;
    }

    boost.active = true;
    boost.lifetime = boost.lifetime_max;
    boost.cooldown = boost.cooldown_max;
    boost.speed_max_old = motion.speed_max;
    boost.acceleration_old = motion.acceleration;

    motion.speed_max = boost.speed_max;
    motion.acceleration = boost.acceleration;
}
