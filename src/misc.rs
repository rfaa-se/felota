use crate::entities::{Entities, EntityIndex};

const MAX_SIZE: usize = 10;

pub struct QuadTree {
    width: f32,
    height: f32,
    nodes: Vec<Node>,
}

pub struct Node {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    eidxs: Vec<EntityIndex>,
}

impl QuadTree {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            nodes: vec![Node {
                x: 0.0,
                y: 0.0,
                width,
                height,
                eidxs: Vec::new(),
            }],
        }
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.nodes.push(Node {
            x: 0.0,
            y: 0.0,
            width: self.width,
            height: self.height,
            eidxs: Vec::new(),
        });
    }

    pub fn add(&mut self, eidx: EntityIndex, entities: &Entities) {
        if self.nodes[0].eidxs.len() == MAX_SIZE {
            // TODO: expand
        }
    }

    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }
}

impl Node {
    pub fn eidxs(&self) -> &[EntityIndex] {
        &self.eidxs
    }
}
