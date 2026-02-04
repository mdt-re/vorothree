use vorothree::{BoundingBox, Tessellation, Wall};
use vorothree::geometries::{SphereGeometry, PlaneGeometry, TrefoilKnotGeometry};
use rand::Rng;

#[test]
fn test_two_cells_neighbors() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 10.0, 10.0, 10.0);
    let mut tess = Tessellation::new(bounds, 1, 1, 1);

    // Two points: one at left, one at right
    let generators = vec![
        2.5, 5.0, 5.0, // 0
        7.5, 5.0, 5.0, // 1
    ];

    tess.set_generators(&generators);
    tess.calculate();

    let c0 = tess.get(0).unwrap();
    let c1 = tess.get(1).unwrap();

    // c0 should have neighbor 1
    assert!(c0.face_neighbors().contains(&1));
    // c1 should have neighbor 0
    assert!(c1.face_neighbors().contains(&0));
}

#[test]
fn test_neighbor_reciprocity_random() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 30.0, 30.0, 30.0);
    let mut tess = Tessellation::new(bounds, 5, 5, 5);

    // 27 random points
    let mut rng = rand::thread_rng();
    let mut generators = Vec::new();
    for _ in 0..27 {
        generators.push(rng.gen_range(0.0..30.0));
        generators.push(rng.gen_range(0.0..30.0));
        generators.push(rng.gen_range(0.0..30.0));
    }

    tess.set_generators(&generators);
    tess.calculate();

    let count = tess.count_cells();
    for i in 0..count {
        let cell = tess.get(i).unwrap();
        let neighbors = cell.face_neighbors();

        for &n_id in &neighbors {
            if n_id >= 0 {
                let n_idx = n_id as usize;
                let neighbor_cell = tess.get(n_idx).unwrap();
                let neighbor_neighbors = neighbor_cell.face_neighbors();

                if !neighbor_neighbors.contains(&(i as i32)) {
                    println!("Reciprocity fail: Cell {} has neighbor {}, but {} has neighbors {:?}", i, n_id, n_id, neighbor_neighbors);
                }

                assert!(
                    neighbor_neighbors.contains(&(i as i32)),
                    "Cell {} claims neighbor {}, but {} does not claim {}",
                    i, n_id, n_id, i
                );
            } else {
                // Boundary or wall
                assert!(n_id < 0);
            }
        }
    }
}

#[test]
fn test_neighbor_reciprocity_half_sphere() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 30.0, 30.0, 30.0);
    let mut tess = Tessellation::new(bounds, 5, 5, 5);

    // 50 random points
    let mut rng = rand::thread_rng();
    let mut generators = Vec::new();
    for _ in 0..50 {
        generators.push(rng.gen_range(0.0..30.0));
        generators.push(rng.gen_range(0.0..30.0));
        generators.push(rng.gen_range(0.0..30.0));
    }

    tess.set_generators(&generators);
    
    // Sphere wall: center (15, 15, 15), radius 12. ID -10.
    tess.add_wall(Wall::new(-10, Box::new(SphereGeometry::new([15.0, 15.0, 15.0], 12.0))));
    
    // Plane wall: point (15, 15, 15), normal (1, 0, 0). Keeps x >= 15. ID -11.
    tess.add_wall(Wall::new(-11, Box::new(PlaneGeometry::new([15.0, 15.0, 15.0], [1.0, 0.0, 0.0]))));

    tess.calculate();

    let count = tess.count_cells();
    for i in 0..count {
        let cell = tess.get(i).unwrap();
        
        // Skip empty cells (fully clipped by walls)
        if cell.vertices().is_empty() {
            continue;
        }

        let neighbors = cell.face_neighbors();
        println!("Cell {}: neighbors {:?}", i, neighbors);

        for &n_id in &neighbors {
            if n_id >= 0 {
                let n_idx = n_id as usize;
                let neighbor_cell = tess.get(n_idx).unwrap();
                let neighbor_neighbors = neighbor_cell.face_neighbors();

                if !neighbor_neighbors.contains(&(i as i32)) {
                    println!("Reciprocity fail: Cell {} with neighbors {:?} claims neighbor {}, but {} has neighbors {:?}", i, neighbors, n_id, n_id, neighbor_neighbors);
                }

                assert!(
                    neighbor_neighbors.contains(&(i as i32)),
                    "Cell {} claims neighbor {}, but {} does not claim {}",
                    i, n_id, n_id, i
                );
            } else {
                // Boundary or wall
                assert!(n_id < 0);
            }
        }
    }
}

#[test]
fn test_neighbor_reciprocity_sphere_small() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 20.0, 20.0, 20.0);
    let mut tess = Tessellation::new(bounds, 5, 5, 5);

    // Sphere wall: center (10, 10, 10), radius 8. ID -10.
    tess.add_wall(Wall::new(-10, Box::new(SphereGeometry::new([10.0, 10.0, 10.0], 8.0))));

    // Generate 10 random points inside the valid region
    tess.random_generators(40);

    tess.calculate();

    let count = tess.count_cells();
    for i in 0..count {
        let cell = tess.get(i).unwrap();
        
        // Skip empty cells (fully clipped by walls)
        if cell.vertices().is_empty() {
            continue;
        }

        let neighbors = cell.face_neighbors();
        println!("Cell {}: neighbors {:?}", i, neighbors);

        for &n_id in &neighbors {
            if n_id >= 0 {
                let n_idx = n_id as usize;
                let neighbor_cell = tess.get(n_idx).unwrap();
                let neighbor_neighbors = neighbor_cell.face_neighbors();

                assert!(
                    neighbor_neighbors.contains(&(i as i32)),
                    "Cell {} claims neighbor {}, but {} does not claim {}",
                    i, n_id, n_id, i
                );
            } else {
                assert!(n_id < 0);
            }
        }
    }
}

#[test]
fn test_neighbor_reciprocity_trefoil_knot() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 30.0, 30.0, 30.0);
    let mut tess = Tessellation::new(bounds, 5, 5, 5);

    // 100 random points
    let mut rng = rand::thread_rng();
    let mut generators = Vec::new();
    for _ in 0..100 {
        generators.push(rng.gen_range(0.0..30.0));
        generators.push(rng.gen_range(0.0..30.0));
        generators.push(rng.gen_range(0.0..30.0));
    }

    tess.set_generators(&generators);
    
    // Trefoil knot centered in box
    tess.add_wall(Wall::new(-15, Box::new(TrefoilKnotGeometry::new([15.0, 15.0, 15.0], 4.0, 2.0, 100))));

    tess.calculate();

    let count = tess.count_cells();
    for i in 0..count {
        let cell = tess.get(i).unwrap();
        
        // Skip empty cells (fully clipped by walls)
        if cell.vertices().is_empty() {
            continue;
        }

        let neighbors = cell.face_neighbors();

        for &n_id in &neighbors {
            if n_id >= 0 {
                let n_idx = n_id as usize;
                let neighbor_cell = tess.get(n_idx).unwrap();
                let neighbor_neighbors = neighbor_cell.face_neighbors();

                if !neighbor_neighbors.contains(&(i as i32)) {
                    println!("Reciprocity fail: Cell {} with neighbors {:?} claims neighbor {}, but {} has neighbors {:?}", i, neighbors, n_id, n_id, neighbor_neighbors);
                }

                assert!(
                    neighbor_neighbors.contains(&(i as i32)),
                    "Cell {} claims neighbor {}, but {} does not claim {}",
                    i, n_id, n_id, i
                );
            } else {
                // Boundary or wall
                assert!(n_id < 0);
            }
        }
    }
}