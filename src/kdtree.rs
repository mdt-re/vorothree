#[derive(Clone, Copy, Debug)]
struct KdNode {
    min: [f64; 3],
    max: [f64; 3],
    left: u32, // u32::MAX if leaf
    right: u32,
    // Leaf data: indices[start..end]
    start: u32,
    end: u32,
    // Internal node data
    split_val: f64,
    axis: u8,
}

pub struct KdTree {
    nodes: Vec<KdNode>,
    indices: Vec<usize>,
}

impl KdTree {
    pub fn new() -> Self {
        KdTree {
            nodes: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn build(&mut self, generators: &[f64]) {
        let count = generators.len() / 3;
        self.indices = (0..count).collect();
        self.nodes.clear();
        
        if count == 0 {
            return;
        }

        // Reserve memory to avoid reallocations
        // A balanced tree has 2*N nodes roughly
        self.nodes.reserve(count * 2);

        self.build_recursive(0, count, generators);
    }

    fn build_recursive(&mut self, start: usize, end: usize, generators: &[f64]) -> u32 {
        let count = end - start;
        
        // Compute bounding box for this range
        let mut min = [f64::INFINITY; 3];
        let mut max = [f64::NEG_INFINITY; 3];
        
        for i in start..end {
            let idx = self.indices[i];
            let gx = generators[idx * 3];
            let gy = generators[idx * 3 + 1];
            let gz = generators[idx * 3 + 2];
            
            if gx < min[0] { min[0] = gx; }
            if gx > max[0] { max[0] = gx; }
            if gy < min[1] { min[1] = gy; }
            if gy > max[1] { max[1] = gy; }
            if gz < min[2] { min[2] = gz; }
            if gz > max[2] { max[2] = gz; }
        }

        // Leaf condition: small number of points
        if count <= 16 {
            let node_idx = self.nodes.len() as u32;
            self.nodes.push(KdNode {
                min,
                max,
                left: u32::MAX,
                right: u32::MAX,
                start: start as u32,
                end: end as u32,
                split_val: 0.0,
                axis: 0,
            });
            return node_idx;
        }

        // Split
        let axis = if (max[0] - min[0]) >= (max[1] - min[1]) && (max[0] - min[0]) >= (max[2] - min[2]) {
            0
        } else if (max[1] - min[1]) >= (max[2] - min[2]) {
            1
        } else {
            2
        };

        // Median split
        let mid = start + count / 2;
        let (_, _, _) = self.indices[start..end].select_nth_unstable_by(count / 2, |&a, &b| {
            let va = generators[a * 3 + axis];
            let vb = generators[b * 3 + axis];
            va.partial_cmp(&vb).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        let mid_idx = self.indices[mid];
        let split_val = generators[mid_idx * 3 + axis];

        let left = self.build_recursive(start, mid, generators);
        let right = self.build_recursive(mid, end, generators);

        let node_idx = self.nodes.len() as u32;
        self.nodes.push(KdNode {
            min,
            max,
            left,
            right,
            start: 0,
            end: 0,
            split_val,
            axis: axis as u8,
        });
        node_idx
    }

    pub fn query<F>(&self, point: [f64; 3], mut max_dist_sq: f64, generators: &[f64], callback: &mut F) 
    where F: FnMut(usize, f64) -> f64 {
        if self.nodes.is_empty() { return; }
        // Root is the last node pushed in our recursive build
        let root_idx = (self.nodes.len() - 1) as u32;
        self.query_recursive(root_idx, point, &mut max_dist_sq, generators, callback);
    }

    fn query_recursive<F>(&self, node_idx: u32, point: [f64; 3], max_dist_sq: &mut f64, generators: &[f64], callback: &mut F)
    where F: FnMut(usize, f64) -> f64 {
        let node = &self.nodes[node_idx as usize];

        // Pruning: check distance from point to node bounding box
        let mut d2 = 0.0;
        for i in 0..3 {
            let v = point[i];
            if v < node.min[i] { d2 += (node.min[i] - v).powi(2); }
            else if v > node.max[i] { d2 += (v - node.max[i]).powi(2); }
        }
        
        // If the box is further than our search radius (4 * max_dist_sq), skip.
        // Note: Voronoi clipping requires checking points within 2 * radius, so squared distance 4 * radius^2.
        if d2 > 4.0 * *max_dist_sq {
            return;
        }

        // Leaf
        if node.left == u32::MAX {
            for i in node.start..node.end {
                let idx = self.indices[i as usize];
                *max_dist_sq = callback(idx, *max_dist_sq);
            }
            return;
        }

        // Internal
        let axis = node.axis as usize;
        let diff = point[axis] - node.split_val;
        
        // Visit nearest child first
        let (first, second) = if diff <= 0.0 { (node.left, node.right) } else { (node.right, node.left) };
        
        self.query_recursive(first, point, max_dist_sq, generators, callback);
        
        // Check if we need to visit the second child
        // The plane distance is diff^2.
        if diff * diff <= 4.0 * *max_dist_sq {
            self.query_recursive(second, point, max_dist_sq, generators, callback);
        }
    }
}