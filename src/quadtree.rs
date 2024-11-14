use std::collections::HashSet;

use crate::entities::{Entities, EntityIndex};
use raylib::prelude::*;

const MAX_SIZE: usize = 4;
const MAX_DEPTH: u8 = 8;

// TODO: could probably solve this in a better way instead of having boxed stuff

pub struct QuadTree {
    pub root: Node,
    initial: Rectangle,
}

pub struct Node {
    pub node_type: NodeType,
    dimension: Rectangle,
    depth: u8,
}

pub enum NodeType {
    Leaf(Vec<EntityIndex>),
    Branch([Box<Node>; 4]),
}

impl QuadTree {
    pub fn new(width: i32, height: i32) -> Self {
        let initial = Rectangle {
            x: 0.0,
            y: 0.0,
            width: width as f32,
            height: height as f32,
        };

        Self {
            root: Node {
                node_type: NodeType::Leaf(Vec::new()),
                dimension: initial,
                depth: 0,
            },
            initial,
        }
    }

    pub fn reset(&mut self) {
        self.root = Node {
            node_type: NodeType::Leaf(Vec::new()),
            dimension: self.initial,
            depth: 0,
        };
    }

    pub fn add(&mut self, eid: usize, entities: &Entities) {
        let eidx = match entities.entity(eid) {
            Some(eidx) => eidx,
            None => return,
        };

        let bounds = bounds(eidx, entities);

        self.root.add(eidx, bounds, entities);
    }

    pub fn get(&self, area: &Rectangle, entities: &Entities) -> HashSet<EntityIndex> {
        let mut v = HashSet::new();

        // get potential entities
        self.root.get(area, &mut v);

        // remove entities not included within the area
        v.retain(|x| bounds(*x, entities).check_collision_recs(area));

        v
    }

    pub fn draw(&self, r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>) {
        draw_node(&self.root, r);

        fn draw_node(node: &Node, r: &mut RaylibMode2D<RaylibTextureMode<RaylibDrawHandle>>) {
            match &node.node_type {
                NodeType::Leaf(_) => {
                    r.draw_rectangle_lines_ex(node.dimension, 1.0, Color::GREEN);
                }
                NodeType::Branch(nodes) => {
                    for node in nodes {
                        draw_node(&node, r);
                    }
                }
            };
        }
    }
}

fn bounds(eidx: EntityIndex, entities: &Entities) -> Rectangle {
    match eidx {
        EntityIndex::Triship(idx) => &entities.triships[idx].entity.body.polygon,
        EntityIndex::Projectile(idx) => &entities.projectiles[idx].entity.body.polygon,
        EntityIndex::Torpedo(idx) => &entities.torpedoes[idx].entity.body.polygon,
        _ => panic!("bounds {:?}", eidx),
    }
    .bounds_meld
    .new
}

impl Node {
    fn add(&mut self, eidx: EntityIndex, bounds: Rectangle, entities: &Entities) {
        match &mut self.node_type {
            NodeType::Leaf(ents) => {
                if !self.dimension.check_collision_recs(&bounds) {
                    return;
                }

                if self.depth != MAX_DEPTH && ents.len() == MAX_SIZE {
                    self.divide(entities);
                    self.add(eidx, bounds, entities);

                    return;
                }

                ents.push(eidx);
            }
            NodeType::Branch(nodes) => {
                for node in nodes {
                    node.add(eidx, bounds, entities);
                }
            }
        };
    }

    fn get(&self, area: &Rectangle, entities: &mut HashSet<EntityIndex>) {
        match &self.node_type {
            NodeType::Leaf(ents) => {
                if self.dimension.check_collision_recs(&area) {
                    for ent in ents {
                        entities.insert(*ent);
                    }
                }
            }
            NodeType::Branch(nodes) => {
                for node in nodes {
                    node.get(area, entities);
                }
            }
        }
    }

    fn divide(&mut self, entities: &Entities) {
        let ents = match &self.node_type {
            NodeType::Leaf(ents) => ents,
            NodeType::Branch(_) => return,
        };

        let x = self.dimension.x;
        let y = self.dimension.y;
        let width = self.dimension.width / 2.0;
        let height = self.dimension.height / 2.0;
        let depth = self.depth + 1;
        let mut nodes = [
            Box::new(Node {
                node_type: NodeType::Leaf(Vec::new()),
                dimension: Rectangle {
                    x,
                    y,
                    width,
                    height,
                },
                depth,
            }),
            Box::new(Node {
                node_type: NodeType::Leaf(Vec::new()),
                dimension: Rectangle {
                    x: x + width,
                    y,
                    width,
                    height,
                },
                depth,
            }),
            Box::new(Node {
                node_type: NodeType::Leaf(Vec::new()),
                dimension: Rectangle {
                    x: x + width,
                    y: y + height,
                    width,
                    height,
                },
                depth,
            }),
            Box::new(Node {
                node_type: NodeType::Leaf(Vec::new()),
                dimension: Rectangle {
                    x,
                    y: y + height,
                    width,
                    height,
                },
                depth,
            }),
        ];

        for eidx in ents {
            let bounds = bounds(*eidx, entities);

            for node in nodes.iter_mut() {
                node.add(*eidx, bounds, entities);
            }
        }

        self.node_type = NodeType::Branch(nodes);
    }
}
