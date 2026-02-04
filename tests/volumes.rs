use vorothree::{BoundingBox, Tessellation, Wall};
use vorothree::geometries::{SphereGeometry, CylinderGeometry};

fn generate_grid(n: usize, size: f64) -> Vec<f64> {
    let mut generators = Vec::with_capacity(n * n * n * 3);
    let step = size / n as f64;
    let offset = step / 2.0;
    
    for x in 0..n {
        for y in 0..n {
            for z in 0..n {
                generators.push(x as f64 * step + offset);
                generators.push(y as f64 * step + offset);
                generators.push(z as f64 * step + offset);
            }
        }
    }
    generators
}

#[test]
fn test_sphere_volume() {
    let size = 10.0;
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, size, size, size);
    let mut tess = Tessellation::new(bounds, 5, 5, 5);

    // 20^3 = 8000 points for better precision
    let generators = generate_grid(20, size);
    tess.set_generators(&generators);

    let r = 4.0;
    // Sphere centered in the box
    tess.add_wall(Wall::new(-1, Box::new(SphereGeometry::new([size/2.0, size/2.0, size/2.0], r))));
    tess.calculate();

    let total_volume: f64 = (0..tess.count_cells())
        .map(|i| tess.get(i).unwrap().volume())
        .sum();

    let expected_volume = 4.0 / 3.0 * std::f64::consts::PI * r.powi(3);
    
    // Error tolerance 1%
    let error = (total_volume - expected_volume).abs() / expected_volume;
    println!("Sphere Volume: Got {:.4}, Expected {:.4}, Error {:.4}%", total_volume, expected_volume, error * 100.0);
    assert!(error < 0.01, "Volume error too high: {:.4}%", error * 100.0);
}

#[test]
fn test_cylinder_volume() {
    let size = 10.0;
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, size, size, size);
    let mut tess = Tessellation::new(bounds, 5, 5, 5);

    let generators = generate_grid(20, size);
    tess.set_generators(&generators);

    let r = 4.0;
    // Cylinder centered, axis Z
    tess.add_wall(Wall::new(-2, Box::new(CylinderGeometry::new([size/2.0, size/2.0, size/2.0], [0.0, 0.0, 1.0], r))));
    tess.calculate();

    let total_volume: f64 = (0..tess.count_cells())
        .map(|i| tess.get(i).unwrap().volume())
        .sum();

    // Volume = Area * Height. Height is limited by the box (size).
    let expected_volume = std::f64::consts::PI * r.powi(2) * size;
    
    let error = (total_volume - expected_volume).abs() / expected_volume;
    println!("Cylinder Volume: Got {:.4}, Expected {:.4}, Error {:.4}%", total_volume, expected_volume, error * 100.0);
    assert!(error < 0.01, "Volume error too high: {:.4}%", error * 100.0);
}