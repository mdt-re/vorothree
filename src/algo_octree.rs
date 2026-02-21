use crate::bounds::BoundingBox;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use crate::tessellation::SpatialAlgorithm;

#[derive(Clone)]
struct Point {
    index: usize,
    x: f64,
    y: f64,
    z: f64,
}

/// A spatial partitioning structure based on an Octree.
///
/// This structure recursively subdivides the 3D space into eight octants.
/// It is particularly efficient for non-uniform distributions of points,
/// as it adapts the depth of the tree to the local density of points.
pub struct AlgorithmOctree {
    bounds: BoundingBox,
    capacity: usize,
    points: Vec<Point>,
    children: Option<Box<[AlgorithmOctree; 8]>>,
}

impl AlgorithmOctree {
    /// Creates a new `AlgorithmOctree` with the specified bounds and capacity.
    ///
    /// # Arguments
    ///
    /// * `bounds` - The spatial boundaries of the octree.
    /// * `capacity` - The maximum number of points a leaf node can hold before subdividing.
    pub fn new(bounds: BoundingBox, capacity: usize) -> AlgorithmOctree {
        AlgorithmOctree {
            bounds,
            capacity,
            points: Vec::new(),
            children: None,
        }
    }

    /// Inserts a point into the octree.
    ///
    /// Returns `true` if the point was successfully inserted (i.e., it is within bounds),
    /// or `false` otherwise.
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

    /// Clears all points from the octree, resetting it to an empty state.
    pub fn clear(&mut self) {
        self.points.clear();
        self.children = None;
    }

    /// Returns an iterator that yields points in the octree ordered by distance from the query point.
    ///
    /// This iterator uses a priority queue to traverse the octree, ensuring that
    /// closer points (or octants containing closer points) are visited first.
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
            AlgorithmOctree::new(BoundingBox::new(min_x, min_y, min_z, mid_x, mid_y, mid_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(mid_x, min_y, min_z, max_x, mid_y, mid_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(min_x, mid_y, min_z, mid_x, max_y, mid_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(mid_x, mid_y, min_z, max_x, max_y, mid_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(min_x, min_y, mid_z, mid_x, mid_y, max_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(mid_x, min_y, mid_z, max_x, mid_y, max_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(min_x, mid_y, mid_z, mid_x, max_y, max_z), self.capacity),
            AlgorithmOctree::new(BoundingBox::new(mid_x, mid_y, mid_z, max_x, max_y, max_z), self.capacity),
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

impl SpatialAlgorithm for AlgorithmOctree {
    fn set_generators(&mut self, generators: &[f64], _bounds: &BoundingBox) {
        self.clear();
        let count = generators.len() / 3;
        for i in 0..count {
            self.insert(i, generators[i*3], generators[i*3+1], generators[i*3+2]);
        }
    }

    fn update_generator(&mut self, _index: usize, _old_pos: &[f64], _new_pos: &[f64], _bounds: &BoundingBox) {
        // AlgorithmOctree doesn't support efficient single updates easily without removal support.
        // For now, we might need to rebuild or implement remove.
        // Given the current API usage, full rebuild is often acceptable or we can implement remove later.
        // For this refactor, we'll leave it as a no-op or full rebuild if critical, but typically set_generators is used.
        // TODO: Implement efficient update
    }

    fn visit_neighbors<F>(
        &self,
        generators: &[f64],
        index: usize,
        pos: [f64; 3],
        max_dist_sq: &mut f64,
        mut visitor: F,
    ) where
        F: FnMut(usize, [f64; 3], f64) -> f64,
    {
        // AlgorithmOctree nearest_iter yields neighbors. We can iterate until we exceed max_dist_sq?
        // The generic calculate loop checks distance, so we just need to feed candidates.
        // nearest_iter is good because it yields closest first.
        for j in self.nearest_iter(pos[0], pos[1], pos[2]) {
            if index == j { continue; }
            let ox = generators[j * 3];
            let oy = generators[j * 3 + 1];
            let oz = generators[j * 3 + 2];

            let dx = ox - pos[0];
            let dy = oy - pos[1];
            let dz = oz - pos[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;

            if dist_sq > 4.0 * *max_dist_sq {
                break;
            }

            *max_dist_sq = visitor(j, [ox, oy, oz], *max_dist_sq);
        }
    }
}

struct SearchItem<'a> {
    dist_sq: f64,
    node: Option<&'a AlgorithmOctree>,
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

/// An iterator that yields points from the octree in order of increasing distance.
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