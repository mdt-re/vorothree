use vorothree::{BoundingBox, Tessellation};

fn main() {
    // Initialize Rayon explicitly so thread creation (clone3) happens
    // before the heavy calculation we want to profile.
    rayon::ThreadPoolBuilder::new().build_global().unwrap();

    // Define bounds for the simulation
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);

    // Create the tessellation instance with a spatial grid (20x20x20)
    // Adjusting grid size affects the binning performance
    let mut tess = Tessellation::new(bounds, 20, 20, 20);

    // Generate a large number of random points to stress the algorithm
    // 10,000 points is usually enough to get a good profile
    tess.random_generators(100000);

    // Run the calculation (this is the hot path)
    tess.calculate();
}