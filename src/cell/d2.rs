use crate::bounds::BoundingBox;
use crate::bounds::box_side;
use crate::cell::Cell;

/// Scratch buffer to reuse allocations during clipping.
#[derive(Default, Clone)]
pub struct Cell2DScratch {
    vertices: Vec<f64>,
    neighbors: Vec<i32>,
    dists: Vec<f64>,
}

/// A 2D Voronoi cell represented as a polygon.
#[derive(Clone)]
pub struct Cell2D {
    pub(crate) id: usize,
    // Flat array of vertices [x, y, x, y, ...]
    pub(crate) vertices: Vec<f64>,
    // Neighbor ID for each edge. edge_neighbors[i] corresponds to edge starting at vertices[2*i]
    pub(crate) edge_neighbors: Vec<i32>,
}

impl Cell2D {
    pub fn new(id: usize, bounds: BoundingBox<2>) -> Cell2D {
        let vertices = vec![
            bounds.min[0], bounds.min[1], // 0: Bottom-Left
            bounds.max[0], bounds.min[1], // 1: Bottom-Right
            bounds.max[0], bounds.max[1], // 2: Top-Right
            bounds.min[0], bounds.max[1], // 3: Top-Left
        ];

        let edge_neighbors = vec![
            box_side(1, false), // 0->1 (Bottom / Y-Min)
            box_side(0, true),  // 1->2 (Right / X-Max)
            box_side(1, true),  // 2->3 (Top / Y-Max)
            box_side(0, false), // 3->0 (Left / X-Min)
        ];

        Cell2D {
            id,
            vertices,
            edge_neighbors,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn vertices(&self) -> Vec<f64> {
        self.vertices.clone()
    }

    pub fn edge_neighbors(&self) -> Vec<i32> {
        self.edge_neighbors.clone()
    }

    pub fn area(&self) -> f64 {
        let n = self.vertices.len() / 2;
        if n < 3 { return 0.0; }
        
        let mut area = 0.0;
        for i in 0..n {
            let j = (i + 1) % n;
            let xi = self.vertices[i * 2];
            let yi = self.vertices[i * 2 + 1];
            let xj = self.vertices[j * 2];
            let yj = self.vertices[j * 2 + 1];
            area += xi * yj - xj * yi;
        }
        (area * 0.5).abs()
    }

    pub fn centroid(&self) -> [f64; 2] {
        let n = self.vertices.len() / 2;
        if n < 3 { return [0.0, 0.0]; }

        let mut cx = 0.0;
        let mut cy = 0.0;
        let mut area = 0.0;

        for i in 0..n {
            let j = (i + 1) % n;
            let xi = self.vertices[i * 2];
            let yi = self.vertices[i * 2 + 1];
            let xj = self.vertices[j * 2];
            let yj = self.vertices[j * 2 + 1];

            let cross = xi * yj - xj * yi;
            area += cross;
            cx += (xi + xj) * cross;
            cy += (yi + yj) * cross;
        }

        if area.abs() < 1e-9 {
            return [0.0, 0.0];
        }

        let factor = 1.0 / (3.0 * area);
        [cx * factor, cy * factor]
    }
    
    fn clip_with_scratch(&mut self, point: &[f64; 2], normal: &[f64; 2], neighbor_id: i32, scratch: &mut Cell2DScratch, generator: Option<&[f64; 2]>) -> (bool, f64) {
        let px = point[0];
        let py = point[1];
        let nx = normal[0];
        let ny = normal[1];

        let num_verts = self.vertices.len() / 2;
        if num_verts < 3 { return (false, 0.0); }

        scratch.dists.clear();
        scratch.dists.reserve(num_verts);
        
        let mut all_inside = true;
        let mut all_outside = true;

        for i in 0..num_verts {
            let vx = self.vertices[i * 2];
            let vy = self.vertices[i * 2 + 1];
            let d = (vx - px) * nx + (vy - py) * ny;
            scratch.dists.push(d);

            if d > 1e-9 {
                all_inside = false;
            } else if d < -1e-9 {
                all_outside = false;
            }
        }

        if all_inside { return (false, 0.0); }
        if all_outside {
            self.vertices.clear();
            self.edge_neighbors.clear();
            return (true, 0.0);
        }

        scratch.vertices.clear();
        scratch.neighbors.clear();
        let mut max_d2 = 0.0;

        for i in 0..num_verts {
            let j = (i + 1) % num_verts;
            
            let d_i = scratch.dists[i];
            let d_j = scratch.dists[j];
            let neighbor = self.edge_neighbors[i];
            
            if d_i <= 1e-9 {
                // V_i is inside
                scratch.vertices.push(self.vertices[i * 2]);
                scratch.vertices.push(self.vertices[i * 2 + 1]);
                
                if let Some(g) = generator {
                    let dx = self.vertices[i * 2] - g[0];
                    let dy = self.vertices[i * 2 + 1] - g[1];
                    let d2 = dx * dx + dy * dy;
                    if d2 > max_d2 { max_d2 = d2; }
                }

                if d_j <= 1e-9 {
                    // V_j is inside: Keep edge
                    scratch.neighbors.push(neighbor);
                } else {
                    // V_j is outside: Clip
                    let t = d_i / (d_i - d_j);
                    let xi = self.vertices[i * 2];
                    let yi = self.vertices[i * 2 + 1];
                    let xj = self.vertices[j * 2];
                    let yj = self.vertices[j * 2 + 1];
                    
                    let ix = xi + t * (xj - xi);
                    let iy = yi + t * (yj - yi);
                    
                    // Add intersection point
                    // The edge from V_i to I inherits neighbor
                    scratch.neighbors.push(neighbor);
                    
                    scratch.vertices.push(ix);
                    scratch.vertices.push(iy);
                    
                    if let Some(g) = generator {
                        let dx = ix - g[0];
                        let dy = iy - g[1];
                        let d2 = dx * dx + dy * dy;
                        if d2 > max_d2 { max_d2 = d2; }
                    }
                    
                    // The next edge will start at I and go to the next intersection (or vertex).
                    // This is the clipping edge.
                    scratch.neighbors.push(neighbor_id);
                }
            } else {
                // V_i is outside
                if d_j <= 1e-9 {
                    // V_j is inside: Entering
                    let t = d_i / (d_i - d_j);
                    let xi = self.vertices[i * 2];
                    let yi = self.vertices[i * 2 + 1];
                    let xj = self.vertices[j * 2];
                    let yj = self.vertices[j * 2 + 1];
                    
                    let ix = xi + t * (xj - xi);
                    let iy = yi + t * (yj - yi);
                    
                    scratch.vertices.push(ix);
                    scratch.vertices.push(iy);
                    
                    if let Some(g) = generator {
                        let dx = ix - g[0];
                        let dy = iy - g[1];
                        let d2 = dx * dx + dy * dy;
                        if d2 > max_d2 { max_d2 = d2; }
                    }
                    
                    // The edge from I to V_j inherits neighbor
                    scratch.neighbors.push(neighbor);
                }
                // Else V_j outside: Skip
            }
        }

        std::mem::swap(&mut self.vertices, &mut scratch.vertices);
        std::mem::swap(&mut self.edge_neighbors, &mut scratch.neighbors);
        (true, max_d2)
    }
}

impl Cell<2> for Cell2D {
    type Scratch = Cell2DScratch;

    fn new(id: usize, bounds: BoundingBox<2>) -> Self {
        Cell2D::new(id, bounds)
    }

    fn clip(
        &mut self,
        point: &[f64; 2],
        normal: &[f64; 2],
        neighbor_id: i32,
        scratch: &mut Self::Scratch,
        generator: Option<&[f64; 2]>,
    ) -> (bool, f64) {
        self.clip_with_scratch(point, normal, neighbor_id, scratch, generator)
    }

    fn max_radius_sq(&self, center: &[f64; 2]) -> f64 {
        let gx = center[0];
        let gy = center[1];
        let mut max_d2 = 0.0;
        for k in 0..self.vertices.len() / 2 {
            let dx = self.vertices[k * 2] - gx;
            let dy = self.vertices[k * 2 + 1] - gy;
            let d2 = dx * dx + dy * dy;
            if d2 > max_d2 {
                max_d2 = d2;
            }
        }
        max_d2
    }

    fn centroid(&self) -> [f64; 2] {
        self.centroid()
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell2d_box() {
        let bounds = BoundingBox::new([0.0, 0.0], [1.0, 1.0]);
        let cell = Cell2D::new(0, bounds);
        
        assert!((cell.area() - 1.0).abs() < 1e-6);
        let c = cell.centroid();
        assert!((c[0] - 0.5).abs() < 1e-6);
        assert!((c[1] - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_cell2d_clip() {
        let bounds = BoundingBox::new([0.0, 0.0], [1.0, 1.0]);
        let mut cell = Cell2D::new(0, bounds);
        let mut scratch = Cell2DScratch::default();
        
        // Clip with x > 0.5 (normal pointing to x+)
        // Point (0.5, 0.5), Normal (1.0, 0.0)
        // Keeps x <= 0.5
        cell.clip_with_scratch(&[0.5, 0.5], &[1.0, 0.0], 10, &mut scratch, None);
        
        assert!((cell.area() - 0.5).abs() < 1e-6);
        let c = cell.centroid();
        assert!((c[0] - 0.25).abs() < 1e-6);
    }
}