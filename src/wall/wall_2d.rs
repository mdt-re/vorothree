use super::WallGeometry;

/// A wall defined by a line in 2D.
///
/// The line partitions space into two regions: valid (inside) and invalid (outside).
/// The normal vector points towards the valid region.
#[derive(Debug)]
pub struct LineGeometry {
    /// A point on the line.
    pub point: [f64; 2],
    /// The normal vector of the line, pointing towards the valid region.
    pub normal: [f64; 2], // Points IN (towards valid region)
}

impl LineGeometry {
    pub fn new(point: [f64; 2], normal: [f64; 2]) -> Self {
        let len = (normal[0] * normal[0] + normal[1] * normal[1]).sqrt();
        let n = if len == 0.0 { [0.0, 1.0] } else { [normal[0] / len, normal[1] / len] };
        Self { point, normal: n }
    }
}

impl WallGeometry<2> for LineGeometry {
    fn contains(&self, point: &[f64; 2]) -> bool {
        let dx = point[0] - self.point[0];
        let dy = point[1] - self.point[1];
        (dx * self.normal[0] + dy * self.normal[1]) >= 0.0
    }

    fn cut(&self, _generator: &[f64; 2], callback: &mut dyn FnMut([f64; 2], [f64; 2])) {
        // Normal points IN, clip expects normal pointing OUT.
        callback(self.point, [-self.normal[0], -self.normal[1]]);
    }

    fn is_planar(&self) -> bool {
        true
    }
}

/// A wall defined by a circle in 2D.
///
/// The valid region is inside the circle.
#[derive(Debug)]
pub struct CircleGeometry {
    pub center: [f64; 2],
    pub radius: f64,
}

impl CircleGeometry {
    pub fn new(center: [f64; 2], radius: f64) -> Self {
        Self { center, radius }
    }
}

impl WallGeometry<2> for CircleGeometry {
    fn contains(&self, point: &[f64; 2]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        (dx * dx + dy * dy) <= self.radius * self.radius
    }

    fn cut(&self, generator: &[f64; 2], callback: &mut dyn FnMut([f64; 2], [f64; 2])) {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dist = (dx * dx + dy * dy).sqrt();

        if dist == 0.0 { return; }

        let scale = self.radius / dist;
        let px = self.center[0] + dx * scale;
        let py = self.center[1] + dy * scale;

        let nx = dx / dist;
        let ny = dy / dist;

        callback([px, py], [nx, ny]);
    }
}

/// A wall defined by a convex polygon in 2D.
#[derive(Debug)]
pub struct ConvexPolygonGeometry2D {
    pub lines: Vec<([f64; 2], [f64; 2])>, // point, normal (OUT)
}

impl ConvexPolygonGeometry2D {
    pub fn new(points: &[f64], normals: &[f64]) -> Self {
        let count = points.len() / 2;
        let mut lines = Vec::with_capacity(count);
        for i in 0..count {
            lines.push((
                [points[i*2], points[i*2+1]],
                [normals[i*2], normals[i*2+1]]
            ));
        }
        Self { lines }
    }
    
    pub fn new_regular(center: [f64; 2], radius: f64, sides: usize) -> Self {
        let mut lines = Vec::with_capacity(sides);
        let angle_step = std::f64::consts::TAU / sides as f64;
        // Inradius for regular polygon
        let inradius = radius * (std::f64::consts::PI / sides as f64).cos();
        
        for i in 0..sides {
            let angle = i as f64 * angle_step;
            let nx = angle.cos();
            let ny = angle.sin();
            
            let px = center[0] + nx * inradius;
            let py = center[1] + ny * inradius;
            
            lines.push(([px, py], [nx, ny]));
        }
        Self { lines }
    }
}

impl WallGeometry<2> for ConvexPolygonGeometry2D {
    fn contains(&self, point: &[f64; 2]) -> bool {
        for (p, n) in &self.lines {
            let dx = point[0] - p[0];
            let dy = point[1] - p[1];
            if dx * n[0] + dy * n[1] > 0.0 {
                return false;
            }
        }
        true
    }

    fn cut(&self, _generator: &[f64; 2], callback: &mut dyn FnMut([f64; 2], [f64; 2])) {
        for (p, n) in &self.lines {
            callback(*p, *n);
        }
    }

    fn is_planar(&self) -> bool {
        true
    }
}

/// A wall defined by an annulus (washer) in 2D.
#[derive(Debug)]
pub struct AnnulusGeometry {
    pub center: [f64; 2],
    pub inner_radius: f64,
    pub outer_radius: f64,
}

impl AnnulusGeometry {
    pub fn new(center: [f64; 2], inner_radius: f64, outer_radius: f64) -> Self {
        Self { center, inner_radius, outer_radius }
    }
}

impl WallGeometry<2> for AnnulusGeometry {
    fn contains(&self, point: &[f64; 2]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let d2 = dx * dx + dy * dy;
        d2 >= self.inner_radius * self.inner_radius && d2 <= self.outer_radius * self.outer_radius
    }

    fn cut(&self, generator: &[f64; 2], callback: &mut dyn FnMut([f64; 2], [f64; 2])) {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dist = (dx * dx + dy * dy).sqrt();
        
        if dist == 0.0 { return; }
        
        let dir_x = dx / dist;
        let dir_y = dy / dist;
        
        // Outer circle (concave boundary)
        let p_outer_x = self.center[0] + dir_x * self.outer_radius;
        let p_outer_y = self.center[1] + dir_y * self.outer_radius;
        // Normal points OUT of valid region (away from center)
        callback([p_outer_x, p_outer_y], [dir_x, dir_y]);
        
        // Inner circle (convex boundary)
        let p_inner_x = self.center[0] + dir_x * self.inner_radius;
        let p_inner_y = self.center[1] + dir_y * self.inner_radius;
        // Normal points OUT of valid region (towards center)
        callback([p_inner_x, p_inner_y], [-dir_x, -dir_y]);
    }
}

/// A wall defined by a cubic bezier curve with thickness in 2D.
#[derive(Debug)]
pub struct CubicBezierGeometry2D {
    pub samples: Vec<[f64; 2]>,
    pub radius: f64,
    pub closed: bool,
}

impl CubicBezierGeometry2D {
    pub fn new(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], radius: f64, resolution: usize, closed: bool) -> Self {
        let mut samples = Vec::with_capacity(resolution + 1);
        for i in 0..=resolution {
            let t = i as f64 / resolution as f64;
            samples.push(Self::calculate_point(p0, p1, p2, p3, t));
        }
        Self { samples, radius, closed }
    }
    
    fn calculate_point(p0: [f64; 2], p1: [f64; 2], p2: [f64; 2], p3: [f64; 2], t: f64) -> [f64; 2] {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        [
            mt3 * p0[0] + 3.0 * mt2 * t * p1[0] + 3.0 * mt * t2 * p2[0] + t3 * p3[0],
            mt3 * p0[1] + 3.0 * mt2 * t * p1[1] + 3.0 * mt * t2 * p2[1] + t3 * p3[1],
        ]
    }
    
    fn get_closest_point(&self, point: &[f64; 2]) -> [f64; 2] {
        if self.samples.is_empty() { return [0.0, 0.0]; }
        let mut min_dist_sq = f64::MAX;
        let mut closest_pt = self.samples[0];
        let n = self.samples.len();
        let limit = if self.closed { n } else { n - 1 };
        
        for i in 0..limit {
            let p0 = self.samples[i];
            let p1 = self.samples[(i + 1) % n];
            
            let v = [p1[0] - p0[0], p1[1] - p0[1]];
            let w = [point[0] - p0[0], point[1] - p0[1]];
            
            let c1 = w[0]*v[0] + w[1]*v[1];
            let c2 = v[0]*v[0] + v[1]*v[1];
            let t = if c2 <= 0.0 { 0.0 } else { (c1 / c2).clamp(0.0, 1.0) };
            
            let proj = [p0[0] + v[0] * t, p0[1] + v[1] * t];
            let dx = point[0] - proj[0];
            let dy = point[1] - proj[1];
            let d2 = dx*dx + dy*dy;
            
            if d2 < min_dist_sq {
                min_dist_sq = d2;
                closest_pt = proj;
            }
        }
        closest_pt
    }
}

impl WallGeometry<2> for CubicBezierGeometry2D {
    fn contains(&self, point: &[f64; 2]) -> bool {
        let closest = self.get_closest_point(point);
        let dist_sq = (point[0] - closest[0]).powi(2) + (point[1] - closest[1]).powi(2);
        dist_sq <= self.radius.powi(2)
    }

    fn cut(&self, generator: &[f64; 2], callback: &mut dyn FnMut([f64; 2], [f64; 2])) {
        let closest = self.get_closest_point(generator);
        let dx = generator[0] - closest[0];
        let dy = generator[1] - closest[1];
        let dist = (dx*dx + dy*dy).sqrt();
        
        if dist == 0.0 { return; }
        
        let nx = dx / dist;
        let ny = dy / dist;
        
        let px = closest[0] + nx * self.radius;
        let py = closest[1] + ny * self.radius;
        
        callback([px, py], [nx, ny]);
    }
}
