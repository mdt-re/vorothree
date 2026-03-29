use crate::bounds::BoundingBox;
use crate::bounds::box_side;
use crate::cell::Cell;
use std::collections::HashMap;

/// A facet of a 4D cell (a 3D polyhedron).
#[derive(Clone, Debug)]
struct Facet {
    /// The faces that make up the boundary of this facet.
    /// Each face is a list of vertex indices.
    faces: Vec<Vec<usize>>,
    /// The neighbor ID associated with this facet.
    neighbor: i32,
}

/// Scratch buffer to reuse allocations during clipping.
#[derive(Default, Clone)]
pub struct Cell4DScratch {
    dists: Vec<f64>,
    new_vertices: Vec<f64>,
    vertex_map: Vec<Option<usize>>,
    // Map from (min_idx, max_idx) of an edge to the new vertex index
    edge_map: HashMap<(usize, usize), usize>,
    // Map from start_vertex to end_vertex for lid reconstruction
    lid_segments: HashMap<usize, usize>,
}

/// A 4D Voronoi cell represented as a polytope.
#[derive(Clone)]
pub struct Cell4D {
    pub(crate) id: usize,
    // Flat array of vertices [x, y, z, w, x, y, z, w, ...]
    pub(crate) vertices: Vec<f64>,
    // The 3D facets that bound the 4D cell.
    facets: Vec<Facet>,
}

impl Cell4D {
    pub fn new(id: usize, bounds: BoundingBox<4>) -> Cell4D {
        // Generate 16 vertices for the hypercube
        let mut vertices = Vec::with_capacity(16 * 4);
        for i in 0..16 {
            vertices.push(if (i & 1) == 0 { bounds.min[0] } else { bounds.max[0] });
            vertices.push(if (i & 2) == 0 { bounds.min[1] } else { bounds.max[1] });
            vertices.push(if (i & 4) == 0 { bounds.min[2] } else { bounds.max[2] });
            vertices.push(if (i & 8) == 0 { bounds.min[3] } else { bounds.max[3] });
        }

        // Helper to create a face (ordered loop of vertices)
        // For a square face varying in dims u and v, with fixed dims at base:
        // 0 -> du -> du+dv -> dv
        let make_face = |base: usize, du: usize, dv: usize| -> Vec<usize> {
            vec![base, base + du, base + du + dv, base + dv]
        };

        let mut facets = Vec::with_capacity(8);

        // 8 Facets of a hypercube (Cubes)
        // Each facet fixes one dimension (min or max) and varies the other 3.
        // Dims: 0=x, 1=y, 2=z, 3=w. Strides: 1, 2, 4, 8.
        for dim in 0..4 {
            let stride = 1 << dim;
            // Facet at min (bit=0) and max (bit=1)
            for side in 0..2 {
                let base = if side == 1 { stride } else { 0 };
                let neighbor = box_side(dim, side == 1);
                
                let mut faces = Vec::with_capacity(6);
                // A cube has 6 faces. Fix one of the remaining 3 dims.
                for d1 in 0..4 {
                    if d1 == dim { continue; }
                    let s1 = 1 << d1;
                    // The other two dims
                    let mut others = Vec::new();
                    for d2 in 0..4 {
                        if d2 != dim && d2 != d1 { others.push(1 << d2); }
                    }
                    let s2 = others[0];
                    let s3 = others[1];

                    // Face at min and max of d1
                    faces.push(make_face(base, s3, s2)); // Reverse winding for min face
                    faces.push(make_face(base + s1, s2, s3));
                }
                facets.push(Facet { faces, neighbor });
            }
        }

        Cell4D {
            id,
            vertices,
            facets,
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn vertices(&self) -> Vec<f64> {
        self.vertices.clone()
    }

    pub fn clip_with_scratch(
        &mut self,
        point: &[f64; 4],
        normal: &[f64; 4],
        neighbor_id: i32,
        scratch: &mut Cell4DScratch,
        generator: Option<&[f64; 4]>,
    ) -> (bool, f64) {
        let px = point[0];
        let py = point[1];
        let pz = point[2];
        let pw = point[3];
        let nx = normal[0];
        let ny = normal[1];
        let nz = normal[2];
        let nw = normal[3];

        let num_verts = self.vertices.len() / 4;
        scratch.dists.clear();
        scratch.dists.reserve(num_verts);

        let mut all_inside = true;
        let mut all_outside = true;

        // 1. Calculate distances
        for i in 0..num_verts {
            let vx = self.vertices[i * 4];
            let vy = self.vertices[i * 4 + 1];
            let vz = self.vertices[i * 4 + 2];
            let vw = self.vertices[i * 4 + 3];
            let d = (vx - px) * nx + (vy - py) * ny + (vz - pz) * nz + (vw - pw) * nw;
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
            self.facets.clear();
            return (true, 0.0);
        }

        // 2. Prepare scratch
        scratch.new_vertices.clear();
        scratch.vertex_map.clear();
        scratch.vertex_map.resize(num_verts, None);
        scratch.edge_map.clear();

        let mut max_d2 = 0.0;

        // Keep inside vertices
        for i in 0..num_verts {
            if scratch.dists[i] <= 1e-9 {
                let new_idx = scratch.new_vertices.len() / 4;
                scratch.new_vertices.push(self.vertices[i * 4]);
                scratch.new_vertices.push(self.vertices[i * 4 + 1]);
                scratch.new_vertices.push(self.vertices[i * 4 + 2]);
                scratch.new_vertices.push(self.vertices[i * 4 + 3]);
                scratch.vertex_map[i] = Some(new_idx);

                if let Some(g) = generator {
                    let dx = self.vertices[i * 4] - g[0];
                    let dy = self.vertices[i * 4 + 1] - g[1];
                    let dz = self.vertices[i * 4 + 2] - g[2];
                    let dw = self.vertices[i * 4 + 3] - g[3];
                    let d2 = dx * dx + dy * dy + dz * dz + dw * dw;
                    if d2 > max_d2 { max_d2 = d2; }
                }
            }
        }

        let mut new_facets = Vec::new();
        let mut lid_faces = Vec::new();

        // 3. Clip each facet
        for facet in &self.facets {
            let mut new_facet_faces = Vec::new();
            scratch.lid_segments.clear();

            for face in &facet.faces {
                let mut new_face = Vec::new();
                let len = face.len();
                
                for i in 0..len {
                    let idx_s = face[i];
                    let idx_e = face[(i + 1) % len];
                    let d_s = scratch.dists[idx_s];
                    let d_e = scratch.dists[idx_e];
                    let s_in = d_s <= 1e-9;
                    let e_in = d_e <= 1e-9;

                    if s_in {
                        if e_in {
                            new_face.push(scratch.vertex_map[idx_e].unwrap());
                        } else {
                            // Start In, End Out -> Intersection
                            let new_idx = Self::get_intersection(idx_s, idx_e, d_s, d_e, &self.vertices, &mut scratch.new_vertices, &mut scratch.edge_map, generator, &mut max_d2);
                            new_face.push(new_idx);
                            // This intersection starts a lid segment for this facet
                            // We need to find the other intersection on this face to complete the segment?
                            // Actually, we can just track segments: (start, end).
                            // The next point added to new_face will be the start of the segment?
                            // No. The polygon is cut. The cut line goes from "Intersection 1" to "Intersection 2".
                            // Intersection 1 is where we go In->Out. Intersection 2 is where we go Out->In.
                            // Wait, if we go In->Out, we add the intersection.
                            // If we go Out->In, we add the intersection.
                            // The segment on the clipping plane connects (In->Out point) to (Out->In point).
                            // Let's store this.
                        }
                    } else if e_in {
                        // Start Out, End In -> Intersection
                        let new_idx = Self::get_intersection(idx_s, idx_e, d_s, d_e, &self.vertices, &mut scratch.new_vertices, &mut scratch.edge_map, generator, &mut max_d2);
                        new_face.push(new_idx);
                        new_face.push(scratch.vertex_map[idx_e].unwrap());
                    }
                }

                if !new_face.is_empty() {
                    new_facet_faces.push(new_face.clone());
                    
                    // Check if this face generated a lid segment
                    // A face is a polygon. If it was cut, it has 2 intersection points (usually).
                    // We find them by checking which vertices in `new_face` are newly created.
                    // But we can just check the transitions above.
                    // Let's refine the loop to capture the segment.
                    // The segment goes from the "In->Out" intersection to the "Out->In" intersection.
                    // In the loop above:
                    // In->Out: we added `new_idx`. This is the START of the cut segment (relative to the face).
                    // Out->In: we added `new_idx`. This is the END of the cut segment.
                    // So we map Start -> End.
                    
                    let mut in_out_idx = None;
                    let mut out_in_idx = None;
                    
                    for i in 0..len {
                        let idx_s = face[i];
                        let idx_e = face[(i + 1) % len];
                        let d_s = scratch.dists[idx_s];
                        let d_e = scratch.dists[idx_e];
                        
                        if d_s <= 1e-9 && d_e > 1e-9 {
                             in_out_idx = Some(Self::get_intersection(idx_s, idx_e, d_s, d_e, &self.vertices, &mut scratch.new_vertices, &mut scratch.edge_map, generator, &mut max_d2));
                        } else if d_s > 1e-9 && d_e <= 1e-9 {
                             out_in_idx = Some(Self::get_intersection(idx_s, idx_e, d_s, d_e, &self.vertices, &mut scratch.new_vertices, &mut scratch.edge_map, generator, &mut max_d2));
                        }
                    }
                    
                    if let (Some(start), Some(end)) = (in_out_idx, out_in_idx) {
                        scratch.lid_segments.insert(end, start);
                    }
                }
            }

            // Reconstruct Lid Face for this facet
            if !scratch.lid_segments.is_empty() {
                let mut lid_face = Vec::new();
                if let Some(&start) = scratch.lid_segments.keys().next() {
                    let mut curr = start;
                    loop {
                        lid_face.push(curr);
                        if let Some(&next) = scratch.lid_segments.get(&curr) {
                            if next == start { break; }
                            curr = next;
                            if lid_face.len() > scratch.lid_segments.len() { break; } // Safety
                        } else {
                            break; // Should not happen for convex
                        }
                    }
                }
                if lid_face.len() >= 3 {
                    new_facet_faces.push(lid_face.clone());
                    
                    // For the global lid facet, we need to reverse the winding of the face
                    // because it is viewed from the other side (the global lid side).
                    let mut reversed_lid = lid_face;
                    reversed_lid.reverse();
                    lid_faces.push(reversed_lid);
                }
            }

            if !new_facet_faces.is_empty() {
                new_facets.push(Facet {
                    faces: new_facet_faces,
                    neighbor: facet.neighbor,
                });
            }
        }

        // 4. Create Global Lid Facet
        if !lid_faces.is_empty() {
            new_facets.push(Facet {
                faces: lid_faces,
                neighbor: neighbor_id,
            });
        }

        self.vertices = scratch.new_vertices.clone();
        self.facets = new_facets;

        (true, max_d2)
    }

    fn get_intersection(
        idx_s: usize,
        idx_e: usize,
        d_s: f64,
        d_e: f64,
        old_vertices: &[f64],
        new_vertices: &mut Vec<f64>,
        edge_map: &mut HashMap<(usize, usize), usize>,
        generator: Option<&[f64; 4]>,
        max_d2: &mut f64,
    ) -> usize {
        let key = if idx_s < idx_e { (idx_s, idx_e) } else { (idx_e, idx_s) };
        if let Some(&idx) = edge_map.get(&key) {
            return idx;
        }

        let t = d_s / (d_s - d_e);
        let vs = &old_vertices[idx_s * 4..];
        let ve = &old_vertices[idx_e * 4..];
        
        let nx = vs[0] + t * (ve[0] - vs[0]);
        let ny = vs[1] + t * (ve[1] - vs[1]);
        let nz = vs[2] + t * (ve[2] - vs[2]);
        let nw = vs[3] + t * (ve[3] - vs[3]);

        let new_idx = new_vertices.len() / 4;
        new_vertices.push(nx);
        new_vertices.push(ny);
        new_vertices.push(nz);
        new_vertices.push(nw);

        if let Some(g) = generator {
            let dx = nx - g[0];
            let dy = ny - g[1];
            let dz = nz - g[2];
            let dw = nw - g[3];
            let d2 = dx * dx + dy * dy + dz * dz + dw * dw;
            if d2 > *max_d2 { *max_d2 = d2; }
        }

        edge_map.insert(key, new_idx);
        new_idx
    }

    pub fn volume(&self) -> f64 {
        let c = self.centroid();
        let mut vol = 0.0;
        for facet in &self.facets {
            // Decompose facet into pyramids (FacetCentroid, Face)
            // First compute facet centroid
            let mut fc = [0.0; 4];
            let mut count = 0.0;
            for face in &facet.faces {
                for &idx in face {
                    fc[0] += self.vertices[idx * 4];
                    fc[1] += self.vertices[idx * 4 + 1];
                    fc[2] += self.vertices[idx * 4 + 2];
                    fc[3] += self.vertices[idx * 4 + 3];
                    count += 1.0;
                }
            }
            if count == 0.0 { continue; }
            fc[0] /= count; fc[1] /= count; fc[2] /= count; fc[3] /= count;

            for face in &facet.faces {
                if face.len() < 3 { continue; }
                // Decompose face into triangles (FaceCentroid, Edge)
                let mut ffc = [0.0; 4];
                for &idx in face {
                    ffc[0] += self.vertices[idx * 4];
                    ffc[1] += self.vertices[idx * 4 + 1];
                    ffc[2] += self.vertices[idx * 4 + 2];
                    ffc[3] += self.vertices[idx * 4 + 3];
                }
                let f_count = face.len() as f64;
                ffc[0] /= f_count; ffc[1] /= f_count; ffc[2] /= f_count; ffc[3] /= f_count;

                for i in 0..face.len() {
                    let idx1 = face[i];
                    let idx2 = face[(i + 1) % face.len()];
                    
                    // Pentatope: C, FC, FFC, V1, V2
                    let v1 = &self.vertices[idx1*4..];
                    let v2 = &self.vertices[idx2*4..];
                    
                    vol += Self::pentatope_volume(&c, &fc, &ffc, v1, v2);
                }
            }
        }
        vol
    }

    fn pentatope_volume(p0: &[f64; 4], p1: &[f64; 4], p2: &[f64; 4], p3: &[f64], p4: &[f64]) -> f64 {
        // V = |Det(v1-v0, v2-v0, v3-v0, v4-v0)| / 24
        let d1 = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2], p1[3]-p0[3]];
        let d2 = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2], p2[3]-p0[3]];
        let d3 = [p3[0]-p0[0], p3[1]-p0[1], p3[2]-p0[2], p3[3]-p0[3]];
        let d4 = [p4[0]-p0[0], p4[1]-p0[1], p4[2]-p0[2], p4[3]-p0[3]];

        // Determinant of 4x4 matrix
        //      | x1 y1 z1 w1 |
        //      | x2 y2 z2 w2 |
        // Det =| x3 y3 z3 w3 |
        //      | x4 y4 z4 w4 |
        
        let det = 
              d1[0] * Self::det3(d2[1], d2[2], d2[3], d3[1], d3[2], d3[3], d4[1], d4[2], d4[3])
            - d1[1] * Self::det3(d2[0], d2[2], d2[3], d3[0], d3[2], d3[3], d4[0], d4[2], d4[3])
            + d1[2] * Self::det3(d2[0], d2[1], d2[3], d3[0], d3[1], d3[3], d4[0], d4[1], d4[3])
            - d1[3] * Self::det3(d2[0], d2[1], d2[2], d3[0], d3[1], d3[2], d4[0], d4[1], d4[2]);
            
        (det / 24.0).abs()
    }

    fn det3(a: f64, b: f64, c: f64, d: f64, e: f64, f: f64, g: f64, h: f64, i: f64) -> f64 {
        a * (e * i - f * h) - b * (d * i - f * g) + c * (d * h - e * g)
    }

    fn max_radius_sq(&self, center: &[f64; 4]) -> f64 {
        let mut max_d2 = 0.0;
        for k in 0..self.vertices.len() / 4 {
            let dx = self.vertices[k * 4] - center[0];
            let dy = self.vertices[k * 4 + 1] - center[1];
            let dz = self.vertices[k * 4 + 2] - center[2];
            let dw = self.vertices[k * 4 + 3] - center[3];
            let d2 = dx * dx + dy * dy + dz * dz + dw * dw;
            if d2 > max_d2 {
                max_d2 = d2;
            }
        }
        max_d2
    }

    pub fn centroid(&self) -> [f64; 4] {
        // Simple average of vertices for convex hull approximation
        // For exact centroid, we need weighted sum of pentatopes, but this is usually sufficient for Voronoi seeds
        let mut c = [0.0; 4];
        let n = self.vertices.len() / 4;
        if n == 0 { return c; }
        
        for i in 0..n {
            c[0] += self.vertices[i * 4];
            c[1] += self.vertices[i * 4 + 1];
            c[2] += self.vertices[i * 4 + 2];
            c[3] += self.vertices[i * 4 + 3];
        }
        let inv_n = 1.0 / n as f64;
        [c[0] * inv_n, c[1] * inv_n, c[2] * inv_n, c[3] * inv_n]
    }

    fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }
}

impl Cell<4> for Cell4D {
    type Scratch = Cell4DScratch;

    fn new(id: usize, bounds: BoundingBox<4>) -> Self {
        Cell4D::new(id, bounds)
    }

    fn clip(
        &mut self,
        point: &[f64; 4],
        normal: &[f64; 4],
        neighbor_id: i32,
        scratch: &mut Self::Scratch,
        generator: Option<&[f64; 4]>,
    ) -> (bool, f64) {
        self.clip_with_scratch(point, normal, neighbor_id, scratch, generator)
    }

    fn max_radius_sq(&self, center: &[f64; 4]) -> f64 {
        self.max_radius_sq(center)
    }

    fn centroid(&self) -> [f64; 4] {
        self.centroid()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}
