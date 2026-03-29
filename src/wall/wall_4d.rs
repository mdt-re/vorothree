use super::WallGeometry;

/// A wall defined by a hyperplane in 4D.
///
/// The hyperplane partitions space into two regions: valid (inside) and invalid (outside).
/// The normal vector points towards the valid region.
#[derive(Debug)]
pub struct Hyperplane4DGeometry {
    /// A point on the hyperplane.
    pub point: [f64; 4],
    /// The normal vector of the hyperplane, pointing towards the valid region.
    pub normal: [f64; 4], // Points IN (towards valid region)
}

impl Hyperplane4DGeometry {
    /// Creates a new `Hyperplane4DGeometry`.
    ///
    /// # Arguments
    ///
    /// * `point` - A point on the hyperplane.
    /// * `normal` - The normal vector of the hyperplane, pointing towards the valid region.
    ///              It will be normalized.
    pub fn new(point: [f64; 4], normal: [f64; 4]) -> Self {
        let len = (normal[0] * normal[0] + normal[1] * normal[1] + normal[2] * normal[2] + normal[3] * normal[3]).sqrt();
        let n = if len == 0.0 { [0.0, 0.0, 0.0, 1.0] } else { [normal[0] / len, normal[1] / len, normal[2] / len, normal[3] / len] };
        Self { point, normal: n }
    }
}

impl WallGeometry<4> for Hyperplane4DGeometry {
    fn contains(&self, point: &[f64; 4]) -> bool {
        let dx = point[0] - self.point[0];
        let dy = point[1] - self.point[1];
        let dz = point[2] - self.point[2];
        let dw = point[3] - self.point[3];
        (dx * self.normal[0] + dy * self.normal[1] + dz * self.normal[2] + dw * self.normal[3]) >= 0.0
    }

    fn cut(&self, _generator: &[f64; 4], callback: &mut dyn FnMut([f64; 4], [f64; 4])) {
        // For a hyperplane wall, the cut is the hyperplane itself.
        // Our normal points IN, but clip expects normal pointing OUT.
        callback(self.point, [-self.normal[0], -self.normal[1], -self.normal[2], -self.normal[3]]);
    }
}

/// A wall defined by a hypersphere (3-sphere) in 4D.
///
/// The valid region is inside the hypersphere.
#[derive(Debug)]
pub struct Spherical4DGeometry {
    /// The center of the hypersphere.
    pub center: [f64; 4],
    /// The radius of the hypersphere.
    pub radius: f64,
}

impl Spherical4DGeometry {
    /// Creates a new `Spherical4DGeometry`.
    ///
    /// # Arguments
    ///
    /// * `center` - The center of the hypersphere.
    /// * `radius` - The radius of the hypersphere.
    pub fn new(center: [f64; 4], radius: f64) -> Self {
        Self { center, radius }
    }
}

impl WallGeometry<4> for Spherical4DGeometry {
    fn contains(&self, point: &[f64; 4]) -> bool {
        let dx = point[0] - self.center[0];
        let dy = point[1] - self.center[1];
        let dz = point[2] - self.center[2];
        let dw = point[3] - self.center[3];
        (dx * dx + dy * dy + dz * dz + dw * dw) <= self.radius * self.radius
    }

    fn cut(&self, generator: &[f64; 4], callback: &mut dyn FnMut([f64; 4], [f64; 4])) {
        let dx = generator[0] - self.center[0];
        let dy = generator[1] - self.center[1];
        let dz = generator[2] - self.center[2];
        let dw = generator[3] - self.center[3];
        let dist = (dx * dx + dy * dy + dz * dz + dw * dw).sqrt();

        if dist == 0.0 { return; }

        // Project generator to hypersphere surface
        let scale = self.radius / dist;
        let px = self.center[0] + dx * scale;
        let py = self.center[1] + dy * scale;
        let pz = self.center[2] + dz * scale;
        let pw = self.center[3] + dw * scale;

        // Normal at surface pointing OUT of hypersphere (away from center)
        let nx = dx / dist;
        let ny = dy / dist;
        let nz = dz / dist;
        let nw = dw / dist;

        callback([px, py, pz, pw], [nx, ny, nz, nw]);
    }
}
