use crate::{
    components::Rotatable,
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
}

const ACCELERATE: u8 = 1;
const DECELERATE: u8 = 2;
const ROTATE_LEFT: u8 = 3;
const ROTATE_RIGHT: u8 = 4;

impl Command {
    pub fn execute(&self, entities: &mut Entities, id: usize, forge: &Forge, h: &mut RaylibHandle) {
        let Some(eidx) = entities.entity(id) else {
            return;
        };

        match self {
            Command::Accelerate => handle_accelerate(entities, eidx, forge, h),
            Command::Decelerate => handle_decelerate(entities, eidx, forge, h),
            Command::RotateLeft => handle_rotate_left(entities, eidx, forge, h),
            Command::RotateRight => handle_rotate_right(entities, eidx, forge, h),
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
            (e.body.generation.new.rotation, &mut e.motion)
        }
        _ => return,
    };

    motion.velocity += rotation * motion.acceleration;

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let orignator_velocity = motion.velocity;
        let triship = &entities.triships[idx].entity;

        let body = triship
            .body
            .generation
            .new
            .shape
            .rotated(triship.body.generation.new.rotation);

        // calculate the placement position of the afterburner
        let position = Vector2::new((body.v1.x + body.v3.x) / 2.0, (body.v1.y + body.v3.y) / 2.0);

        // rotate 180 degrees, we want the exhaust to be pointed away from the rotation of the entity
        let exhaust_rotation = Vector2 {
            x: rotation.x * -1.0,
            y: rotation.y * -1.0,
        };

        for exhaust in forge.exhaust_afterburner(position, exhaust_rotation, orignator_velocity, h)
        {
            entities.add(Entity::Exhaust(exhaust));
        }
    }
}

fn handle_decelerate(
    entities: &mut Entities,
    eidx: EntityIndex,
    _forge: &Forge,
    _h: &mut RaylibHandle,
) {
    let (rotation, motion) = match eidx {
        EntityIndex::Triship(idx) => {
            let e = &mut entities.triships[idx].entity;
            (e.body.generation.new.rotation, &mut e.motion)
        }
        _ => return,
    };

    motion.velocity -= rotation * (motion.acceleration / 4.0);

    // spawn exhaust particles if it's a triship
    // TODO: we want two thrusters on each side of the ship
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
            (&mut e.motion, e.body.generation.old.rotation)
        }
        _ => return,
    };

    motion.rotation_speed -= motion.rotation_acceleration;

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let triship = &entities.triships[idx].entity;
        let originator_velocity = triship.motion.velocity;

        let body = triship.body.generation.new.shape.rotated(old_rotation);

        // calculate the placement position of the thruster
        let position = Vector2::new(
            body.v3.x * 0.2 + body.v2.x * 0.8,
            body.v3.y * 0.2 + body.v2.y * 0.8,
        );

        // rotate 270 degrees, we want the exhaust to be pointed to the right of the entity
        let exhaust_rotation = Vector2 {
            x: old_rotation.y * -1.0,
            y: old_rotation.x,
        };

        for exhaust in
            forge.exhaust_thruster_side(position, exhaust_rotation, originator_velocity, h)
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
            (&mut e.motion, e.body.generation.old.rotation)
        }
        _ => return,
    };

    motion.rotation_speed += motion.rotation_acceleration;

    // spawn exhaust particles if it's a triship
    if let EntityIndex::Triship(idx) = eidx {
        let triship = &entities.triships[idx].entity;
        let originator_velocity = triship.motion.velocity;

        let body = triship.body.generation.new.shape.rotated(old_rotation);

        // calculate the placement position of the thruster
        let position = Vector2::new(
            body.v1.x * 0.2 + body.v2.x * 0.8,
            body.v1.y * 0.2 + body.v2.y * 0.8,
        );

        // rotate 90 degrees, we want the exhaust to be pointed to the right of the entity
        let exhaust_rotation = Vector2 {
            x: old_rotation.y,
            y: old_rotation.x * -1.0,
        };

        for exhaust in
            forge.exhaust_thruster_side(position, exhaust_rotation, originator_velocity, h)
        {
            entities.add(Entity::Exhaust(exhaust));
        }
    }
}
