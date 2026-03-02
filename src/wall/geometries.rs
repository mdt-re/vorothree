use super::WallGeometry;

// --- 2D Geometries ---

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

// --- 3D Geometries ---

/// A wall defined by a plane.
///
/// The plane partitions space into two regions: valid (inside) and invalid (outside).
/// The normal vector points towards the valid region.
#[derive(Debug)]
pub struct PlaneGeometry {
    /// A point on the plane.
    pub point: [f64; 3],
    /// The normal vector of the plane, pointing towards the valid region.
    pub normal: [f64; 3], // Points IN (towards valid region)
}

impl PlaneGeometry {
    /// Creates a new `PlaneGeometry`.
    ///
    /// # Arguments
    ///
    /// * `point` - A point on the plane.
    /// * `normal` - The normal vector of the plane, pointing towards the valid region.
    ///              It will be normalized.
    pub fn new(point: [f64; 3], normal: [f64; 3]) -> Self {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2]).sqrt();
        let n = if len == 0.0 { [0.0, 0.0, 1.0] } else { [normal[0] / len, normal[1] / len, normal[2] / len] };
        Self { point, normal: n }
    }
}

impl WallGeometry<3> for PlaneGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.point[0];
        let dy = point[1] - self.point[1];
        let dz = point[2] - self.point[2];
        (dx * self.normal[0] + dy * self.normal[1] + dz * self.normal[2]) >= 0.0
    }

    fn cut(&self, _generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        // For a plane wall, the cut is the plane itself.
        // Our normal points IN, but clip expects normal pointing OUT.
        callback(self.point, [-self.normal[0], -self.normal[1], -self.normal[2]]);
    }
}

/// A wall defined by a sphere.
///
/// The valid region is inside the sphere.
#[derive(Debug)]
pub struct SphereGeometry {
    /// The center of the sphere.
    pub center: [f64; 3],
    /// The radius of the sphere.
    pub radius: f64,
}

impl SphereGeometry {
    /// Creates a new `SphereGeometry`.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the sphere.
    /// * `radius` - The radius of the sphere.
    pub fn new(center: [f64; 3], radius: f64) -> Self {
        Self { center, radius }
    }
}

impl WallGeometry<3> for SphereGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        (dx * dx + dy * dy + dz * dz) <= self.radius * self.radius
    }

    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];
        let dist = (dx * dx + dy * dy + dz * dz).sqrt();

        if dist == 0.0 { return; }

        // Project generator to sphere surface
        let scale = self.radius / dist;
        let px = self.center[0] + dx * scale;
        let py = self.center[1] + dy * scale;
        let pz = self.center[2] + dz * scale;

        // Normal at surface pointing OUT of sphere (away from center)
        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;

        callback([px, py, pz], [nx, ny, nz]);
    }
}

/// A wall defined by an infinite cylinder.
///
/// The valid region is inside the cylinder.
#[derive(Debug)]
pub struct CylinderGeometry {
    /// A point on the cylinder's axis.
    pub center: [f64; 3],
    /// The direction of the cylinder's axis.
    pub axis: [f64; 3],
    /// The radius of the cylinder.
    pub radius: f64,
}

impl CylinderGeometry {
    /// Creates a new `CylinderGeometry`.
    ///
    /// # Arguments
    ///
    /// * `center` - A point on the cylinder's axis.
    /// * `axis` - The direction of the cylinder's axis. It will be normalized.
    /// * `radius` - The radius of the cylinder.
    pub fn new(center: [f64; 3], axis: [f64; 3], radius: f64) -> Self {
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let a = if len == 0.0 { [0.0, 0.0, 1.0] } else { [axis[0] / len, axis[1] / len, axis[2] / len] };
        Self { center, axis: a, radius }
    }
}

impl WallGeometry<3> for CylinderGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        
        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z) <= self.radius * self.radius
    }

    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];

        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        let dist = (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z).sqrt();
        if dist == 0.0 { return; }

        // Project to cylinder surface
        let scale = self.radius / dist;
        let px = self.center[0] + dot * self.axis[0] + perp_x * scale;
        let py = self.center[1] + dot * self.axis[1] + perp_y * scale;
        let pz = self.center[2] + dot * self.axis[2] + perp_z * scale;

        // Normal pointing OUT (away from axis)
        let nx = perp_x / dist;
        let ny = perp_y / dist;
        let nz = perp_z / dist;

        callback([px, py, pz], [nx, ny, nz]);
    }
}

/// A wall defined by an infinite cone.
///
/// The valid region is inside the cone.
#[derive(Debug)]
pub struct ConeGeometry {
    /// The tip (apex) of the cone.
    pub tip: [f64; 3],
    /// The direction of the cone's axis (pointing into the cone).
    pub axis: [f64; 3],
    /// The half-angle of the cone in radians.
    pub angle: f64,
}

impl ConeGeometry {
    /// Creates a new `ConeGeometry`.
    ///
    /// # Arguments
    ///
    /// * `tip` - The tip (apex) of the cone.
    /// * `axis` - The direction of the cone's axis. It will be normalized.
    /// * `angle` - The half-angle of the cone in radians.
    pub fn new(tip: [f64; 3], axis: [f64; 3], angle: f64) -> Self {
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let a = if len == 0.0 { [0.0, 0.0, 1.0] } else { [axis[0] / len, axis[1] / len, axis[2] / len] };
        Self { tip, axis: a, angle }
    }
}

impl WallGeometry<3> for ConeGeometry {
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

/// A wall defined by a torus.
///
/// The valid region is inside the torus tube.
#[derive(Debug)]
pub struct TorusGeometry {
    /// The center of the torus.
    pub center: [f64; 3],
    /// The axis of the torus (perpendicular to the major circle).
    pub axis: [f64; 3],
    /// The radius of the major circle (distance from center to tube center).
    pub major_radius: f64,
    /// The radius of the tube.
    pub minor_radius: f64,
}

impl TorusGeometry {
    /// Creates a new `TorusGeometry`.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the torus.
    /// * `axis` - The axis of the torus. It will be normalized.
    /// * `major_radius` - The radius of the major circle.
    /// * `minor_radius` - The radius of the tube.
    pub fn new(center: [f64; 3], axis: [f64; 3], major_radius: f64, minor_radius: f64) -> Self {
        let len = (axis[0] * axis[0] + axis[1] * axis[1] + axis[2] * axis[2]).sqrt();
        let a = if len == 0.0 { [0.0, 0.0, 1.0] } else { [axis[0] / len, axis[1] / len, axis[2] / len] };
        Self { center, axis: a, major_radius, minor_radius }
    }
}

impl WallGeometry<3> for TorusGeometry {
    fn contains(&self, point: &[f64; 3]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        
        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        let dist_perp = (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z).sqrt();
        
        // Distance to the tube center (which is at distance major_radius from axis)
        let dist_tube = ((dist_perp - self.major_radius).powi(2) + dot.powi(2)).sqrt();
        
        dist_tube <= self.minor_radius
    }

    fn cut(&self, generator: &[f64; 3], callback: &mut dyn FnMut([f64; 3], [f64; 3])) {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];

        let dot = dx * self.axis[0] + dy * self.axis[1] + dz * self.axis[2];
        let perp_x = dx - dot * self.axis[0];
        let perp_y = dy - dot * self.axis[1];
        let perp_z = dz - dot * self.axis[2];
        
        let dist_perp = (perp_x * perp_x + perp_y * perp_y + perp_z * perp_z).sqrt();
        
        // Determine direction from axis to projected point
        let (dir_x, dir_y, dir_z) = if dist_perp < 1e-9 {
            // Singularity on axis: pick an arbitrary perpendicular vector
            let mut tx = 1.0; let mut ty = 0.0; let tz = 0.0;
            if self.axis[0].abs() > 0.9 { tx = 0.0; ty = 1.0; }
            let t_dot = tx * self.axis[0] + ty * self.axis[1] + tz * self.axis[2];
            let ax = tx - t_dot * self.axis[0];
            let ay = ty - t_dot * self.axis[1];
            let az = tz - t_dot * self.axis[2];
            let len = (ax*ax + ay*ay + az*az).sqrt();
            if len == 0.0 { return; }
            (ax/len, ay/len, az/len)
        } else {
            (perp_x / dist_perp, perp_y / dist_perp, perp_z / dist_perp)
        };

        // Closest point on the major circle
        let cx = self.center[0] + dir_x * self.major_radius;
        let cy = self.center[1] + dir_y * self.major_radius;
        let cz = self.center[2] + dir_z * self.major_radius;

        // Vector from C to Generator
        let v_cx = generator[0] - cx;
        let v_cy = generator[1] - cy;
        let v_cz = generator[2] - cz;
        let dist_c = (v_cx*v_cx + v_cy*v_cy + v_cz*v_cz).sqrt();

        if dist_c == 0.0 { return; }

        // Normal pointing OUT (away from C)
        let nx = v_cx / dist_c;
        let ny = v_cy / dist_c;
        let nz = v_cz / dist_c;

        // Point on surface
        let px = cx + nx * self.minor_radius;
        let py = cy + ny * self.minor_radius;
        let pz = cz + nz * self.minor_radius;

        callback([px, py, pz], [nx, ny, nz]);
    }
}

/// A wall defined by a trefoil knot tube.
///
/// The valid region is inside the tube following the knot path.
#[derive(Debug)]
pub struct TrefoilKnotGeometry {
    /// The center of the knot.
    pub center: [f64; 3],
    /// The scale of the knot.
    pub scale: f64,
    /// The radius of the tube.
    pub tube_radius: f64,
    /// Sample points along the knot curve.
    pub samples: Vec<[f64; 3]>,
}

impl TrefoilKnotGeometry {
    /// Creates a new `TrefoilKnotGeometry`.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the knot.
    /// * `scale` - The scale of the knot.
    /// * `tube_radius` - The radius of the tube.
    /// * `resolution` - The number of sample points along the curve.
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

impl WallGeometry<3> for TrefoilKnotGeometry {
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

/// A wall defined by a convex polyhedron.
///
/// The valid region is inside the polyhedron, defined by the intersection of half-spaces.
#[derive(Debug)]
pub struct ConvexPolyhedronGeometry {
    /// The planes defining the faces of the polyhedron.
    /// Each tuple contains a point on the plane and the normal vector pointing OUT of the valid region.
    pub planes: Vec<([f64; 3], [f64; 3])>, // (point, normal) where normal points OUT of the valid region
}

impl ConvexPolyhedronGeometry {
    /// Creates a new `ConvexPolyhedronGeometry` from a list of points and normals.
    ///
    /// # Arguments
    ///
    /// * `points` - A flat array of points on the planes [x1, y1, z1, x2, y2, z2, ...].
    /// * `normals` - A flat array of normal vectors for the planes [nx1, ny1, nz1, ...].
    ///               Normals should point OUT of the valid region.
    pub fn new(points: &[f64], normals: &[f64]) -> Self {
        if points.len() != normals.len() || points.len() % 3 != 0 {
            panic!("Points and normals must have same length and be multiple of 3");
        }
        
        let count = points.len() / 3;
        let mut planes = Vec::with_capacity(count);
        
        for i in 0..count {
            planes.push((
                [points[i*3], points[i*3+1], points[i*3+2]],
                [normals[i*3], normals[i*3+1], normals[i*3+2]]
            ));
        }
        Self { planes }
    }

    /// Creates a regular tetrahedron wall.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the tetrahedron.
    /// * `radius` - The circumradius of the tetrahedron.
    pub fn new_tetrahedron(center: [f64; 3], radius: f64) -> Self {
        // Inradius r = R / 3
        let dist = radius / 3.0;
        
        // Normals (unnormalized) pointing OUT of the faces
        // Corresponds to faces opposite to vertices (1,1,1), (1,-1,-1), (-1,1,-1), (-1,-1,1)
        let base_normals: [[f64; 3]; 4] = [
            [-1.0, -1.0, -1.0],
            [-1.0, 1.0, 1.0],
            [1.0, -1.0, 1.0],
            [1.0, 1.0, -1.0],
        ];

        let mut planes = Vec::with_capacity(4);
        for n in base_normals {
            let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
            let nx = n[0] / len;
            let ny = n[1] / len;
            let nz = n[2] / len;
            
            let px = center[0] + nx * dist;
            let py = center[1] + ny * dist;
            let pz = center[2] + nz * dist;
            planes.push(([px, py, pz], [nx, ny, nz]));
        }

        Self { planes }
    }

    /// Creates a regular hexahedron (cube) wall.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the hexahedron.
    /// * `radius` - The circumradius of the hexahedron.
    pub fn new_hexahedron(center: [f64; 3], radius: f64) -> Self {
        // Inradius r = R / sqrt(3)
        let dist = radius / 3.0f64.sqrt();

        let base_normals = [
            [1.0, 0.0, 0.0], [-1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0], [0.0, -1.0, 0.0],
            [0.0, 0.0, 1.0], [0.0, 0.0, -1.0],
        ];

        let mut planes = Vec::with_capacity(6);
        for n in base_normals {
            let nx = n[0];
            let ny = n[1];
            let nz = n[2];
            
            let px = center[0] + nx * dist;
            let py = center[1] + ny * dist;
            let pz = center[2] + nz * dist;
            planes.push(([px, py, pz], [nx, ny, nz]));
        }

        Self { planes }
    }

    /// Creates a regular octahedron wall.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the octahedron.
    /// * `radius` - The circumradius of the octahedron.
    pub fn new_octahedron(center: [f64; 3], radius: f64) -> Self {
        // Inradius r = R / sqrt(3)
        let dist = radius / 3.0f64.sqrt();

        let mut base_normals: Vec<[f64; 3]> = Vec::with_capacity(8);
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    base_normals.push([x, y, z]);
                }
            }
        }

        let mut planes = Vec::with_capacity(8);
        for n in base_normals {
            let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
            let nx = n[0] / len;
            let ny = n[1] / len;
            let nz = n[2] / len;
            
            let px = center[0] + nx * dist;
            let py = center[1] + ny * dist;
            let pz = center[2] + nz * dist;
            planes.push(([px, py, pz], [nx, ny, nz]));
        }

        Self { planes }
    }

    /// Creates a regular dodecahedron wall.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the dodecahedron.
    /// * `radius` - The circumradius of the dodecahedron.
    pub fn new_dodecahedron(center: [f64; 3], radius: f64) -> Self {
        let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
        // Distance from center to face for a dodecahedron with circumradius R
        // r = R * xi, where xi = sqrt((5 + 2*sqrt(5)) / 15)
        let xi = ((5.0 + 2.0 * 5.0f64.sqrt()) / 15.0).sqrt();
        let dist = radius * xi;

        // Base normals (unnormalized)
        let base_normals = [
            [0.0, phi, 1.0], [0.0, -phi, 1.0], [0.0, phi, -1.0], [0.0, -phi, -1.0],
            [1.0, 0.0, phi], [1.0, 0.0, -phi], [-1.0, 0.0, phi], [-1.0, 0.0, -phi],
            [phi, 1.0, 0.0], [phi, -1.0, 0.0], [-phi, 1.0, 0.0], [-phi, -1.0, 0.0],
        ];

        let mut planes = Vec::with_capacity(12);
        for n in base_normals {
            let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
            let nx = n[0] / len;
            let ny = n[1] / len;
            let nz = n[2] / len;
            
            let px = center[0] + nx * dist;
            let py = center[1] + ny * dist;
            let pz = center[2] + nz * dist;
            planes.push(([px, py, pz], [nx, ny, nz]));
        }

        Self { planes }
    }

    /// Creates a regular icosahedron wall.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the icosahedron.
    /// * `radius` - The circumradius of the icosahedron.
    pub fn new_icosahedron(center: [f64; 3], radius: f64) -> Self {
        let phi = (1.0 + 5.0f64.sqrt()) / 2.0;
        // Ratio of inradius to circumradius for Icosahedron: sqrt((5 + 2*sqrt(5)) / 15)
        let xi = ((5.0 + 2.0 * 5.0f64.sqrt()) / 15.0).sqrt();
        let dist = radius * xi;

        // Normals are vertices of a Dodecahedron
        let one_over_phi = 1.0 / phi;
        let mut base_normals = Vec::with_capacity(20);
        
        // (±1, ±1, ±1)
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                for z in [-1.0, 1.0] {
                    base_normals.push([x, y, z]);
                }
            }
        }
        // (0, ±phi, ±1/phi)
        for y in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                base_normals.push([0.0, y * phi, z * one_over_phi]);
            }
        }
        // (±1/phi, 0, ±phi)
        for x in [-1.0, 1.0] {
            for z in [-1.0, 1.0] {
                base_normals.push([x * one_over_phi, 0.0, z * phi]);
            }
        }
        // (±phi, ±1/phi, 0)
        for x in [-1.0, 1.0] {
            for y in [-1.0, 1.0] {
                base_normals.push([x * phi, y * one_over_phi, 0.0]);
            }
        }

        let mut planes = Vec::with_capacity(20);
        for n in base_normals {
            let len = (n[0]*n[0] + n[1]*n[1] + n[2]*n[2]).sqrt();
            let nx = n[0] / len;
            let ny = n[1] / len;
            let nz = n[2] / len;
            
            let px = center[0] + nx * dist;
            let py = center[1] + ny * dist;
            let pz = center[2] + nz * dist;
            planes.push(([px, py, pz], [nx, ny, nz]));
        }

        Self { planes }
    }
}

impl WallGeometry<3> for ConvexPolyhedronGeometry {
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

/// A wall defined by a tube around a Cubic Bezier curve.
///
/// The valid region is inside the tube following the curve.
#[derive(Debug)]
pub struct CubicBezierGeometry {
    /// Sample points along the curve.
    pub samples: Vec<[f64; 3]>,
    /// The radius of the tube.
    pub tube_radius: f64,
    /// Whether the tube is closed (loops back to start).
    pub closed: bool,
}

impl CubicBezierGeometry {
    /// Creates a new `CubicBezierGeometry`.
    ///
    /// # Arguments
    ///
    /// * `p0` - The start point.
    /// * `p1` - The first control point.
    /// * `p2` - The second control point.
    /// * `p3` - The end point.
    /// * `tube_radius` - The radius of the tube.
    /// * `resolution` - The number of segments to approximate the curve.
    /// * `closed` - Whether the tube should be closed (looping).
    pub fn new(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], tube_radius: f64, resolution: usize, closed: bool) -> Self {
        let mut samples = Vec::with_capacity(resolution + 1);
        for i in 0..=resolution {
            let t = i as f64 / resolution as f64;
            samples.push(Self::calculate_cubic_bezier_point(p0, p1, p2, p3, t));
        }
        Self { samples, tube_radius, closed }
    }

    fn calculate_cubic_bezier_point(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], t: f64) -> [f64; 3] {
        let t2 = t * t;
        let t3 = t2 * t;
        let mt = 1.0 - t;
        let mt2 = mt * mt;
        let mt3 = mt2 * mt;

        [
            mt3 * p0[0] + 3.0 * mt2 * t * p1[0] + 3.0 * mt * t2 * p2[0] + t3 * p3[0],
            mt3 * p0[1] + 3.0 * mt2 * t * p1[1] + 3.0 * mt * t2 * p2[1] + t3 * p3[1],
            mt3 * p0[2] + 3.0 * mt2 * t * p1[2] + 3.0 * mt * t2 * p2[2] + t3 * p3[2],
        ]
    }

    fn get_closest_point(&self, point: &[f64; 3]) -> [f64; 3] {
        if self.samples.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let mut min_dist_sq = f64::MAX;
        let mut closest_pt = self.samples[0];
        let n = self.samples.len();
        
        // Iterate over segments
        let limit = if self.closed { n } else { n - 1 };
        
        for i in 0..limit {
            let p0 = self.samples[i];
            let p1 = self.samples[(i + 1) % n];
            
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

impl WallGeometry<3> for CubicBezierGeometry {
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

/// A wall defined by a tube around a Catmull-Rom spline.
///
/// The valid region is inside the tube following the curve.
#[derive(Debug)]
pub struct CatmullRomGeometry {
    /// Sample points along the curve.
    pub samples: Vec<[f64; 3]>,
    /// The radius of the tube.
    pub tube_radius: f64,
    /// Whether the tube is closed (loops back to start).
    pub closed: bool,
}

impl CatmullRomGeometry {
    pub fn new(points: Vec<[f64; 3]>, tube_radius: f64, resolution: usize, closed: bool) -> Self {
        let mut samples = Vec::with_capacity(resolution + 1);
        if points.len() >= 2 {
            for i in 0..=resolution {
                let t = i as f64 / resolution as f64;
                samples.push(Self::get_point(t, &points, closed));
            }
        }
        Self { samples, tube_radius, closed }
    }

    fn get_point(t: f64, points: &[[f64; 3]], closed: bool) -> [f64; 3] {
        let l = points.len();
        let p = (l as f64 - if closed { 0.0 } else { 1.0 }) * t;
        let mut int_point = p.floor() as isize;
        let weight = p - int_point as f64;

        if closed {
            if int_point > 0 {
                 int_point += 0;
            } else {
                 int_point += (int_point.abs() / l as isize + 1) * l as isize;
            }
        } else if weight == 0.0 && int_point == l as isize - 1 {
            int_point = l as isize - 2;
        }

        let p0;
        let p1;
        let p2;
        let p3;

        if closed || int_point > 0 {
             p0 = points[( (int_point - 1) % l as isize + l as isize) as usize % l];
        } else {
             p0 = [
                 points[0][0] - (points[1][0] - points[0][0]),
                 points[0][1] - (points[1][1] - points[0][1]),
                 points[0][2] - (points[1][2] - points[0][2]),
             ];
        }

        p1 = points[int_point as usize % l];
        p2 = points[(int_point + 1) as usize % l];

        if closed || int_point + 2 < l as isize {
            p3 = points[(int_point + 2) as usize % l];
        } else {
            let last = points[l-1];
            let prev = points[l-2];
            p3 = [
                last[0] - (prev[0] - last[0]),
                last[1] - (prev[1] - last[1]),
                last[2] - (prev[2] - last[2]),
            ];
        }

        let pow = 0.25;
        let mut dt0 = dist_sq(p0, p1).powf(pow);
        let mut dt1 = dist_sq(p1, p2).powf(pow);
        let mut dt2 = dist_sq(p2, p3).powf(pow);

        if dt1 < 1e-4 { dt1 = 1.0; }
        if dt0 < 1e-4 { dt0 = dt1; }
        if dt2 < 1e-4 { dt2 = dt1; }

        let px = init_nonuniform_catmull_rom(p0[0], p1[0], p2[0], p3[0], dt0, dt1, dt2);
        let py = init_nonuniform_catmull_rom(p0[1], p1[1], p2[1], p3[1], dt0, dt1, dt2);
        let pz = init_nonuniform_catmull_rom(p0[2], p1[2], p2[2], p3[2], dt0, dt1, dt2);

        [
            px.calc(weight),
            py.calc(weight),
            pz.calc(weight),
        ]
    }

    fn get_closest_point(&self, point: &[f64; 3]) -> [f64; 3] {
        if self.samples.is_empty() {
            return [0.0, 0.0, 0.0];
        }
        let mut min_dist_sq = f64::MAX;
        let mut closest_pt = self.samples[0];
        let n = self.samples.len();
        
        // Iterate over segments
        let limit = if self.closed { n } else { n - 1 };
        
        for i in 0..limit {
            let p0 = self.samples[i];
            let p1 = self.samples[(i + 1) % n];
            
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

impl WallGeometry<3> for CatmullRomGeometry {
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

fn dist_sq(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

struct CubicPoly {
    c0: f64, c1: f64, c2: f64, c3: f64
}

impl CubicPoly {
    fn calc(&self, t: f64) -> f64 {
        let t2 = t * t;
        let t3 = t2 * t;
        self.c0 + self.c1 * t + self.c2 * t2 + self.c3 * t3
    }
}

fn init_nonuniform_catmull_rom(x0: f64, x1: f64, x2: f64, x3: f64, dt0: f64, dt1: f64, dt2: f64) -> CubicPoly {
    let mut t1 = (x1 - x0) / dt0 - (x2 - x0) / (dt0 + dt1) + (x2 - x1) / dt1;
    let mut t2 = (x2 - x1) / dt1 - (x3 - x1) / (dt1 + dt2) + (x3 - x2) / dt2;

    t1 *= dt1;
    t2 *= dt1;

    let c0 = x1;
    let c1 = t1;
    let c2 = -3.0 * x1 + 3.0 * x2 - 2.0 * t1 - t2;
    let c3 = 2.0 * x1 - 2.0 * x2 + t1 + t2;

    CubicPoly { c0, c1, c2, c3 }
}