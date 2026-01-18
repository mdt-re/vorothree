use crate::WallGeometry;

#[derive(Debug)]
pub struct ConeGeometry {
    pub tip: [f64; 3],
    pub axis: [f64; 3],
    pub angle: f64,
}

impl WallGeometry for ConeGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.tip[0];
        let dy = point[1] - self.tip[1];
        let dz = point[2] - self.tip[2];
        
        let h = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let px = dx - h * self.axis[0];
        let py = dy - h * self.axis[1];
        let pz = dz - h * self.axis[2];
        let r = (px*px + py*py + pz*pz).sqrt();
        
        h >= 0.0 && r <= h * self.angle.tan()
    }

    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        let dx = generator[0] - self.tip[0];
        let dy = generator[1] - self.tip[1];
        let dz = generator[2] - self.tip[2];
        
        let h = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let px = dx - h * self.axis[0];
        let py = dy - h * self.axis[1];
        let pz = dz - h * self.axis[2];
        let r = (px*px + py*py + pz*pz).sqrt();
        
        if r == 0.0 {
            return;
        }
        
        let r_dir_x = px / r;
        let r_dir_y = py / r;
        let r_dir_z = pz / r;
        
        let cos_a = self.angle.cos();
        let sin_a = self.angle.sin();
        
        let dist = r * cos_a - h * sin_a;
        
        let p2d_r = r - dist * cos_a;
        let p2d_h = h + dist * sin_a;
        
        if p2d_h < 0.0 {
            let dist_tip = (dx*dx + dy*dy + dz*dz).sqrt();
            if dist_tip == 0.0 { return; }
            return callback(self.tip, [dx/dist_tip, dy/dist_tip, dz/dist_tip]);
        }
        
        let surf_x = self.tip[0] + p2d_h * self.axis[0] + p2d_r * r_dir_x;
        let surf_y = self.tip[1] + p2d_h * self.axis[1] + p2d_r * r_dir_y;
        let surf_z = self.tip[2] + p2d_h * self.axis[2] + p2d_r * r_dir_z;
        
        let norm_x = cos_a * r_dir_x - sin_a * self.axis[0];
        let norm_y = cos_a * r_dir_y - sin_a * self.axis[1];
        let norm_z = cos_a * r_dir_z - sin_a * self.axis[2];
        
        callback([surf_x, surf_y, surf_z], [norm_x, norm_y, norm_z]);
    }
}

#[derive(Debug)]
pub struct TrefoilKnotGeometry {
    pub center: [f64; 3],
    pub scale: f64,
    pub tube_radius: f64,
    pub samples: Vec<[f64; 3]>,
}

impl TrefoilKnotGeometry {
    pub fn new(center: [f64; 3], scale: f64, tube_radius: f64, resolution: usize) -> Self {
        let mut samples = Vec::with_capacity(resolution);
        for i in 0..resolution {
            let t = (i as f64 / resolution as f64) * std::f64::consts::TAU;
            // Parametric equations for a trefoil knot
            let x = t.sin() + 2.0 * (2.0 * t).sin();
            let y = t.cos() - 2.0 * (2.0 * t).cos();
            let z = -(3.0 * t).sin();
            
            samples.push([
                center[0] + x * scale,
                center[1] + y * scale,
                center[2] + z * scale,
            ]);
        }
        Self { center, scale, tube_radius, samples }
    }

    fn get_closest_point(&self, point: &[f64; 3]) -> [f64; 3] {
        let mut min_dist_sq = f64::MAX;
        let mut closest_pt = self.samples[0];
        let n = self.samples.len();
        
        for i in 0..n {
            let p0 = self.samples[i];
            let p1 = self.samples[(i + 1) % n];
            
            // Project point onto segment p0-p1
            let v = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let w = [point[0] - p0[0], point[1] - p0[1], point[2] - p0[2]];
            
            let c1 = w[0]*v[0] + w[1]*v[1] + w[2]*v[2];
            let c2 = v[0]*v[0] + v[1]*v[1] + v[2]*v[2];
            
            let t = if c2 <= 0.0 { 0.0 } else { (c1 / c2).clamp(0.0, 1.0) };
            
            let proj = [p0[0] + v[0] * t, p0[1] + v[1] * t, p0[2] + v[2] * t];
            let dx = point[0] - proj[0];
            let dy = point[1] - proj[1];
            let dz = point[2] - proj[2];
            let d2 = dx*dx + dy*dy + dz*dz;
            
            if d2 < min_dist_sq {
                min_dist_sq = d2;
                closest_pt = proj;
            }
        }
        closest_pt
    }
}

impl WallGeometry for TrefoilKnotGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let closest = self.get_closest_point(point);
        let dist_sq = (point[0] - closest[0]).powi(2) + (point[1] - closest[1]).powi(2) + (point[2] - closest[2]).powi(2);
        dist_sq <= self.tube_radius.powi(2)
    }

    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        let closest = self.get_closest_point(generator);
        let dist = ((generator[0] - closest[0]).powi(2) + (generator[1] - closest[1]).powi(2) + (generator[2] - closest[2]).powi(2)).sqrt();
        if dist == 0.0 { return; }
        let normal = [(generator[0] - closest[0]) / dist, (generator[1] - closest[1]) / dist, (generator[2] - closest[2]) / dist];
        let surface_point = [closest[0] + normal[0] * self.tube_radius, closest[1] + normal[1] * self.tube_radius, closest[2] + normal[2] * self.tube_radius];
        callback(surface_point, normal);
    }
}

#[derive(Debug)]
pub struct ConvexPolyhedronGeometry {
    pub planes: Vec<([f64; 3], [f64; 3])>, // (point, normal) where normal points OUT of the valid region
}

impl WallGeometry for ConvexPolyhedronGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        for (p, n) in &self.planes {
            let dx = point[0] - p[0];
            let dy = point[1] - p[1];
            let dz = point[2] - p[2];
            if dx * n[0] + dy * n[1] + dz * n[2] > 0.0 {
                return false;
            }
        }
        true
    }

    fn cut(&self, _generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        for (p, n) in &self.planes {
            callback(*p, *n);
        }
    }
}