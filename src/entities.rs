use std::collections::HashMap;

use crate::components::*;

use raylib::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum EntityIndex {
    Triship(usize),
    Projectile(usize),
    Exhaust(usize),
    Explosion(usize),
    Star(usize),
    Torpedo(usize),
}

pub enum Entity {
    Triship(Triship),
    Projectile(Projectile),
    Exhaust(Particle),
    Explosion(Particle),
    Star(Particle),
    Torpedo(Torpedo),
}

pub struct EntityId<T> {
    pub id: usize,
    pub entity: T,
}

pub struct Entities {
    pub triships: Vec<EntityId<Triship>>,
    pub projectiles: Vec<EntityId<Projectile>>,
    pub exhausts: Vec<EntityId<Particle>>,
    pub explosions: Vec<EntityId<Particle>>,
    pub stars: Vec<EntityId<Particle>>,
    pub torpedoes: Vec<EntityId<Torpedo>>,

    id_map: HashMap<usize, EntityIndex>,
    id_free: usize,
}

pub struct Triship {
    pub life: f32,
    pub body: Body<Triangle>,
    pub motion: Motion,
    pub boost: Boost,
    pub cooldown_torpedo: Load,
}

pub struct Projectile {
    pub damage: f32,
    pub body: Body<Rectangle>,
    pub motion: Motion,
    pub owner_id: usize,
    pub life: f32,
}

pub struct Torpedo {
    pub damage: f32,
    pub body: Body<Rectangle>,
    pub motion: Motion,
    pub owner_id: usize,
    pub timer_inactive: u8,
    pub life: f32,
}

pub struct Particle {
    pub random: u8,
    pub lifetime: u8,
    pub body: Body<Vector2>,
    pub motion: Motion,
}

impl Entities {
    pub fn new() -> Self {
        Self {
            triships: Vec::new(),
            projectiles: Vec::new(),
            exhausts: Vec::new(),
            explosions: Vec::new(),
            stars: Vec::new(),
            torpedoes: Vec::new(),

            id_map: HashMap::new(),
            id_free: 0,
        }
    }

    pub fn total(&self) -> usize {
        self.triships.len()
            + self.projectiles.len()
            + self.exhausts.len()
            + self.explosions.len()
            + self.stars.len()
            + self.torpedoes.len()
    }

    pub fn add(&mut self, entity: Entity) -> usize {
        let id = self.id_free;

        self.id_free += 1;

        let eidx = match entity {
            Entity::Triship(entity) => {
                self.triships.push(EntityId { id, entity });
                EntityIndex::Triship(self.triships.len() - 1)
            }
            Entity::Projectile(entity) => {
                self.projectiles.push(EntityId { id, entity });
                EntityIndex::Projectile(self.projectiles.len() - 1)
            }
            Entity::Exhaust(entity) => {
                self.exhausts.push(EntityId { id, entity });
                EntityIndex::Exhaust(self.exhausts.len() - 1)
            }
            Entity::Explosion(entity) => {
                self.explosions.push(EntityId { id, entity });
                EntityIndex::Explosion(self.explosions.len() - 1)
            }
            Entity::Star(entity) => {
                self.stars.push(EntityId { id, entity });
                EntityIndex::Star(self.stars.len() - 1)
            }
            Entity::Torpedo(entity) => {
                self.torpedoes.push(EntityId { id, entity });
                EntityIndex::Torpedo(self.torpedoes.len() - 1)
            }
        };

        self.id_map.insert(id, eidx);

        id
    }

    pub fn entity(&self, id: usize) -> Option<EntityIndex> {
        match self.id_map.get(&id) {
            Some(eidx) => Some(*eidx),
            None => None,
        }
    }

    pub fn kill(&mut self, id: usize) {
        let map = &mut self.id_map;

        if let Some(eidx) = map.get(&id) {
            match eidx {
                EntityIndex::Triship(idx) => swap_dead(&mut self.triships, map, *idx),
                EntityIndex::Projectile(idx) => swap_dead(&mut self.projectiles, map, *idx),
                EntityIndex::Exhaust(idx) => swap_dead(&mut self.exhausts, map, *idx),
                EntityIndex::Explosion(idx) => swap_dead(&mut self.explosions, map, *idx),
                EntityIndex::Star(idx) => swap_dead(&mut self.stars, map, *idx),
                EntityIndex::Torpedo(idx) => swap_dead(&mut self.torpedoes, map, *idx),
            }
        }
    }

    pub fn centroid(&self, id: usize) -> Option<Generation<Vector2>> {
        let Some(eidx) = self.entity(id) else {
            return None;
        };

        let (old, new) = match eidx {
            EntityIndex::Triship(idx) => {
                let gen = &self.triships[idx].entity.body.state;
                (
                    &gen.old.shape as &dyn Centroidable,
                    &gen.new.shape as &dyn Centroidable,
                )
            }
            _ => return None,
        };

        Some(Generation {
            old: old.centroid(),
            new: new.centroid(),
        })
    }
}

fn swap_dead<T>(
    entities: &mut Vec<EntityId<T>>,
    map: &mut HashMap<usize, EntityIndex>,
    idx: usize,
) {
    // remove the dead entity and swap it with the last one
    let dead = entities.swap_remove(idx);

    let Some(dead_ref) = map.remove(&dead.id) else {
        panic!("invalid map, missing id: {}", dead.id);
    };

    // update reference for the swapped entity
    if let Some(swap) = entities.get_mut(idx) {
        map.insert(swap.id, dead_ref);
    }
}
