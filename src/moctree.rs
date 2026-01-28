use crate::bounds::BoundingBox;
use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone)]
struct Point {
    index: usize,
    x: f64,
    y: f64,
    z: f64,
}

pub struct Moctree {
    bounds: BoundingBox,
    capacity: usize,
    points: Vec<Point>,
    children: Option<Box<[Moctree; 8]>>,
}

impl Moctree {
    pub fn new(bounds: BoundingBox, capacity: usize) -> Moctree {
        Moctree {
            bounds,
            capacity,
            points: Vec::new(),
            children: None,
        }
    }

    pub fn insert(&mut self, index: usize, x: f64, y: f64, z: f64) -> bool {
        if !self.contains(x, y, z) {
            return false;
        }

        if self.children.is_none() {
            if self.points.len() < self.capacity {
                self.points.push(Point { index, x, y, z });
                return true;
            }
            
            self.subdivide();
        }

        // If we have children (either existed or just created)
        if let Some(children) = &mut self.children {
            for child in children.iter_mut() {
                if child.insert(index, x, y, z) {
                    return true;
                }
            }
        }

        false
    }

    pub fn clear(&mut self) {
        self.points.clear();
        self.children = None;
    }

    // pub fn query(&self, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Vec<usize> {
    //     let mut results = Vec::new();
    //     self.query_recursive(min_x, min_y, min_z, max_x, max_y, max_z, &mut results);
    //     results
    // }

    pub fn nearest_iter(&self, x: f64, y: f64, z: f64) -> NearestIterator<'_> {
        let mut queue = BinaryHeap::new();
        
        // Calculate distance to root bounds
        let d2 = dist_sq_to_bounds(x, y, z, &self.bounds);
        
        queue.push(SearchItem {
            dist_sq: d2,
            node: Some(self),
            point: None,
        });

        NearestIterator {
            queue,
            query_x: x,
            query_y: y,
            query_z: z,
        }
    }

    // fn query_recursive(&self, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64, results: &mut Vec<usize>) {
    //     if !self.intersects_range(min_x, min_y, min_z, max_x, max_y, max_z) {
    //         return;
    //     }

    //     for p in &self.points {
    //         if p.x >= min_x && p.x <= max_x &&
    //            p.y >= min_y && p.y <= max_y &&
    //            p.z >= min_z && p.z <= max_z {
    //             results.push(p.index);
    //         }
    //     }

    //     if let Some(children) = &self.children {
    //         for child in children.iter() {
    //             child.query_recursive(min_x, min_y, min_z, max_x, max_y, max_z, results);
    //         }
    //     }
    // }

    // fn intersects_range(&self, min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> bool {
    //     self.bounds.max_x >= min_x && self.bounds.min_x <= max_x &&
    //     self.bounds.max_y >= min_y && self.bounds.min_y <= max_y &&
    //     self.bounds.max_z >= min_z && self.bounds.min_z <= max_z
    // }

    fn contains(&self, x: f64, y: f64, z: f64) -> bool {
        x >= self.bounds.min_x && x <= self.bounds.max_x &&
        y >= self.bounds.min_y && y <= self.bounds.max_y &&
        z >= self.bounds.min_z && z <= self.bounds.max_z
    }

    fn subdivide(&mut self) {
        let min_x = self.bounds.min_x;
        let min_y = self.bounds.min_y;
        let min_z = self.bounds.min_z;
        let max_x = self.bounds.max_x;
        let max_y = self.bounds.max_y;
        let max_z = self.bounds.max_z;

        let mid_x = (min_x + max_x) / 2.0;
        let mid_y = (min_y + max_y) / 2.0;
        let mid_z = (min_z + max_z) / 2.0;

        let mut children = Box::new([
            Moctree::new(BoundingBox::new(min_x, min_y, min_z, mid_x, mid_y, mid_z), self.capacity),
            Moctree::new(BoundingBox::new(mid_x, min_y, min_z, max_x, mid_y, mid_z), self.capacity),
            Moctree::new(BoundingBox::new(min_x, mid_y, min_z, mid_x, max_y, mid_z), self.capacity),
            Moctree::new(BoundingBox::new(mid_x, mid_y, min_z, max_x, max_y, mid_z), self.capacity),
            Moctree::new(BoundingBox::new(min_x, min_y, mid_z, mid_x, mid_y, max_z), self.capacity),
            Moctree::new(BoundingBox::new(mid_x, min_y, mid_z, max_x, mid_y, max_z), self.capacity),
            Moctree::new(BoundingBox::new(min_x, mid_y, mid_z, mid_x, max_y, max_z), self.capacity),
            Moctree::new(BoundingBox::new(mid_x, mid_y, mid_z, max_x, max_y, max_z), self.capacity),
        ]);

        let points = std::mem::take(&mut self.points);
        for p in points {
            for child in children.iter_mut() {
                if child.insert(p.index, p.x, p.y, p.z) {
                    break;
                }
            }
        }

        self.children = Some(children);
    }
}

struct SearchItem<'a> {
    dist_sq: f64,
    node: Option<&'a Moctree>,
    point: Option<&'a Point>,
}

impl<'a> PartialEq for SearchItem<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.dist_sq == other.dist_sq
    }
}

impl<'a> Eq for SearchItem<'a> {}

impl<'a> PartialOrd for SearchItem<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse ordering for Min-Heap behavior
        other.dist_sq.partial_cmp(&self.dist_sq)
    }
}

impl<'a> Ord for SearchItem<'a> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

pub struct NearestIterator<'a> {
    queue: BinaryHeap<SearchItem<'a>>,
    query_x: f64,
    query_y: f64,
    query_z: f64,
}

impl<'a> Iterator for NearestIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.queue.pop() {
            if let Some(point) = item.point {
                return Some(point.index);
            }

            if let Some(node) = item.node {
                if let Some(children) = &node.children {
                    for child in children.iter() {
                        let d2 = dist_sq_to_bounds(self.query_x, self.query_y, self.query_z, &child.bounds);
                        self.queue.push(SearchItem {
                            dist_sq: d2,
                            node: Some(child),
                            point: None,
                        });
                    }
                } else {
                    for p in &node.points {
                        let dx = p.x - self.query_x;
                        let dy = p.y - self.query_y;
                        let dz = p.z - self.query_z;
                        let d2 = dx * dx + dy * dy + dz * dz;
                        self.queue.push(SearchItem {
                            dist_sq: d2,
                            node: None,
                            point: Some(p),
                        });
                    }
                }
            }
        }
        None
    }
}

fn dist_sq_to_bounds(x: f64, y: f64, z: f64, b: &BoundingBox) -> f64 {
    let dx = (b.min_x - x).max(0.0).max(x - b.max_x);
    let dy = (b.min_y - y).max(0.0).max(y - b.max_y);
    let dz = (b.min_z - z).max(0.0).max(z - b.max_z);
    dx * dx + dy * dy + dz * dz
}
