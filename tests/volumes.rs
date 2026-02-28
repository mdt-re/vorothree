use vorothree::{BoundingBox, Tessellation, AlgorithmGrid, AlgorithmOctree, CellFaces, CellEdges, Wall, WALL_ID_START};
use vorothree::geometries::{SphereGeometry, CylinderGeometry, ConvexPolyhedronGeometry};

const GRID_SIZE: usize = 20;

fn generate_grid(size: f64) -> Vec<f64> {
    let n = GRID_SIZE;
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

macro_rules! test_volume {
    ($test_name:ident, $cell_type:ty, $algo_ctor:expr, $setup:expr, $expected:expr) => {
        #[test]
        fn $test_name() {
            let size = 10.0;
            let bounds = BoundingBox::new([0.0, 0.0, 0.0], [size, size, size]);
            let algo = $algo_ctor(&bounds);
            let mut tess = Tessellation::<3, $cell_type, _>::new(bounds, algo);

            let generators = generate_grid(size);
            tess.set_generators(&generators);

            $setup(&mut tess, size);
            tess.calculate();

            let total_volume: f64 = (0..tess.count_cells())
                .map(|i| tess.get_cell(i).unwrap().volume())
                .sum();

            let expected_volume = $expected;
            
            let error = (total_volume - expected_volume).abs() / expected_volume;
            println!("{} Volume: Got {:.4}, Expected {:.4}, Error {:.4}%", stringify!($test_name), total_volume, expected_volume, error * 100.0);
            assert!(error < 0.01, "Volume error too high: {:.4}%", error * 100.0);
        }
    };
}

// Sphere Tests
test_volume!(test_sphere_volume_grid_faces, CellFaces, |b| AlgorithmGrid::new(5, 5, 5, b), 
    |tess: &mut Tessellation<3, CellFaces, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(SphereGeometry::new([size/2.0, size/2.0, size/2.0], r))));
    },
    4.0 / 3.0 * std::f64::consts::PI * 4.0f64.powi(3)
);
test_volume!(test_sphere_volume_grid_edges, CellEdges, |b| AlgorithmGrid::new(5, 5, 5, b), 
    |tess: &mut Tessellation<3, CellEdges, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(SphereGeometry::new([size/2.0, size/2.0, size/2.0], r))));
    },
    4.0 / 3.0 * std::f64::consts::PI * 4.0f64.powi(3)
);
test_volume!(test_sphere_volume_octree_faces, CellFaces, |b: &BoundingBox<3>| AlgorithmOctree::new(*b, 16), 
    |tess: &mut Tessellation<3, CellFaces, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(SphereGeometry::new([size/2.0, size/2.0, size/2.0], r))));
    },
    4.0 / 3.0 * std::f64::consts::PI * 4.0f64.powi(3)
);
test_volume!(test_sphere_volume_octree_edges, CellEdges, |b: &BoundingBox<3>| AlgorithmOctree::new(*b, 16), 
    |tess: &mut Tessellation<3, CellEdges, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(SphereGeometry::new([size/2.0, size/2.0, size/2.0], r))));
    },
    4.0 / 3.0 * std::f64::consts::PI * 4.0f64.powi(3)
);

// Cylinder Tests
test_volume!(test_cylinder_volume_grid_faces, CellFaces, |b| AlgorithmGrid::new(5, 5, 5, b), 
    |tess: &mut Tessellation<3, CellFaces, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(CylinderGeometry::new([size/2.0, size/2.0, size/2.0], [0.0, 0.0, 1.0], r))));
    },
    std::f64::consts::PI * 4.0f64.powi(2) * 10.0
);
test_volume!(test_cylinder_volume_grid_edges, CellEdges, |b| AlgorithmGrid::new(5, 5, 5, b), 
    |tess: &mut Tessellation<3, CellEdges, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(CylinderGeometry::new([size/2.0, size/2.0, size/2.0], [0.0, 0.0, 1.0], r))));
    },
    std::f64::consts::PI * 4.0f64.powi(2) * 10.0
);
test_volume!(test_cylinder_volume_octree_faces, CellFaces, |b: &BoundingBox<3>| AlgorithmOctree::new(*b, 16), 
    |tess: &mut Tessellation<3, CellFaces, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(CylinderGeometry::new([size/2.0, size/2.0, size/2.0], [0.0, 0.0, 1.0], r))));
    },
    std::f64::consts::PI * 4.0f64.powi(2) * 10.0
);
test_volume!(test_cylinder_volume_octree_edges, CellEdges, |b: &BoundingBox<3>| AlgorithmOctree::new(*b, 16), 
    |tess: &mut Tessellation<3, CellEdges, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(CylinderGeometry::new([size/2.0, size/2.0, size/2.0], [0.0, 0.0, 1.0], r))));
    },
    std::f64::consts::PI * 4.0f64.powi(2) * 10.0
);

// Dodecahedron Tests
fn dodecahedron_volume(radius: f64) -> f64 {
    let xi = ((5.0 + 2.0 * 5.0f64.sqrt()) / 15.0).sqrt();
    let dist = radius * xi; // inradius
    let a = 2.0 * dist * 10.0f64.sqrt() / (25.0 + 11.0 * 5.0f64.sqrt()).sqrt();
    (15.0 + 7.0 * 5.0f64.sqrt()) / 4.0 * a.powi(3)
}

test_volume!(test_dodecahedron_volume_grid_faces, CellFaces, |b| AlgorithmGrid::new(5, 5, 5, b), 
    |tess: &mut Tessellation<3, CellFaces, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(ConvexPolyhedronGeometry::new_dodecahedron([size/2.0, size/2.0, size/2.0], r))));
    },
    dodecahedron_volume(4.0)
);
test_volume!(test_dodecahedron_volume_grid_edges, CellEdges, |b| AlgorithmGrid::new(5, 5, 5, b), 
    |tess: &mut Tessellation<3, CellEdges, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(ConvexPolyhedronGeometry::new_dodecahedron([size/2.0, size/2.0, size/2.0], r))));
    },
    dodecahedron_volume(4.0)
);
test_volume!(test_dodecahedron_volume_octree_faces, CellFaces, |b: &BoundingBox<3>| AlgorithmOctree::new(*b, 16), 
    |tess: &mut Tessellation<3, CellFaces, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(ConvexPolyhedronGeometry::new_dodecahedron([size/2.0, size/2.0, size/2.0], r))));
    },
    dodecahedron_volume(4.0)
);
test_volume!(test_dodecahedron_volume_octree_edges, CellEdges, |b: &BoundingBox<3>| AlgorithmOctree::new(*b, 16), 
    |tess: &mut Tessellation<3, CellEdges, _>, size: f64| {
        let r = 4.0;
        tess.add_wall(Wall::new(WALL_ID_START, Box::new(ConvexPolyhedronGeometry::new_dodecahedron([size/2.0, size/2.0, size/2.0], r))));
    },
    dodecahedron_volume(4.0)
);