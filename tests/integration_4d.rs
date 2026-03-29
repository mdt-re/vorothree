use vorothree::{BoundingBox, Cell4D, Algorithm4DGrid, Tessellation, Cell};

#[test]
fn test_cell4d_metrics() {
    // Create a 10x10x10x10 hypercube
    let bounds = BoundingBox::new([0.0, 0.0, 0.0, 0.0], [10.0, 10.0, 10.0, 10.0]);
    let cell = Cell4D::new(0, bounds);

    // Test Volume
    // 10^4 = 10000
    let vol = cell.volume();
    assert!((vol - 10000.0).abs() < 1e-6, "Expected volume 10000, got {}", vol);

    // Test Centroid
    // Center should be at (5, 5, 5, 5)
    let c = cell.centroid();
    assert!((c[0] - 5.0).abs() < 1e-6, "Centroid X mismatch");
    assert!((c[1] - 5.0).abs() < 1e-6, "Centroid Y mismatch");
    assert!((c[2] - 5.0).abs() < 1e-6, "Centroid Z mismatch");
    assert!((c[3] - 5.0).abs() < 1e-6, "Centroid W mismatch");
}

#[test]
fn test_tessellation_4d_workflow() {
    let bounds = BoundingBox::new([0.0, 0.0, 0.0, 0.0], [10.0, 10.0, 10.0, 10.0]);
    let mut tess = Tessellation::<4, Cell4D, _>::new(bounds, Algorithm4DGrid::new(5, 5, 5, 5, &bounds));

    // Two points splitting the domain along X
    let points = vec![
        2.5, 5.0, 5.0, 5.0,
        7.5, 5.0, 5.0, 5.0,
    ];

    tess.set_generators(&points);
    tess.calculate();

    assert_eq!(tess.count_cells(), 2);

    let total_vol: f64 = (0..2).map(|i| tess.get_cell(i).unwrap().volume()).sum();
    assert!((total_vol - 10000.0).abs() < 1e-3, "Total volume should be 10000, got {}", total_vol);
}

#[test]
fn test_cell4d_clip_volume() {
    // Manual clip test
    let bounds = BoundingBox::new([0.0, 0.0, 0.0, 0.0], [10.0, 10.0, 10.0, 10.0]);
    let mut cell = Cell4D::new(0, bounds);
    let mut scratch = <Cell4D as Cell<4>>::Scratch::default();

    // Clip with plane x=5 (normal pointing to +x)
    // Point (5, 5, 5, 5), Normal (1, 0, 0, 0)
    // Keeps x <= 5
    cell.clip(&[5.0, 5.0, 5.0, 5.0], &[1.0, 0.0, 0.0, 0.0], 1, &mut scratch, None);

    let vol = cell.volume();
    assert!((vol - 5000.0).abs() < 1e-3, "Clipped volume should be 5000, got {}", vol);
}