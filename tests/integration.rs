use vorothree::{BoundingBox, CellFaces, TessellationGrid};

#[test]
fn test_cell_metrics() {
    // Create a 10x20x30 box
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 10.0, 20.0, 30.0);
    let cell = CellFaces::new(0, bounds);

    // Test Volume
    // 10 * 20 * 30 = 6000
    let vol = cell.volume();
    assert!((vol - 6000.0).abs() < 1e-6, "Expected volume 6000, got {}", vol);

    // Test Centroid
    // Center should be at (5, 10, 15)
    let c = cell.centroid();
    assert!((c[0] - 5.0).abs() < 1e-6, "Centroid X mismatch");
    assert!((c[1] - 10.0).abs() < 1e-6, "Centroid Y mismatch");
    assert!((c[2] - 15.0).abs() < 1e-6, "Centroid Z mismatch");
}

#[test]
fn test_tessellation_workflow() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    let mut tess = TessellationGrid::new(bounds, 10, 10, 10);

    let points = vec![
        10.0, 10.0, 10.0,
        90.0, 90.0, 90.0,
    ];

    tess.set_generators(&points);
    tess.calculate();

    assert_eq!(tess.count_generators(), 2);
    assert_eq!(tess.count_cells(), 2);

    let c0 = tess.get(0).expect("Should have cell 0");
    let c1 = tess.get(1).expect("Should have cell 1");

    assert_eq!(c0.id(), 0);
    assert_eq!(c1.id(), 1);

    let total_vol = c0.volume() + c1.volume();
    assert!((total_vol - 1_000_000.0).abs() < 1e-3, "Total volume should be 1,000,000, got {}", total_vol);
}

#[test]
fn test_tessellation_cells_octet() {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    let mut tess = TessellationGrid::new(bounds, 10, 10, 10);

    let points = vec![
        25.0, 25.0, 25.0,
        25.0, 25.0, 75.0,
        25.0, 75.0, 25.0,
        25.0, 75.0, 75.0,
        75.0, 25.0, 25.0,
        75.0, 25.0, 75.0,
        75.0, 75.0, 25.0,
        75.0, 75.0, 75.0        
    ];

    tess.set_generators(&points);
    tess.calculate();

    assert_eq!(tess.count_generators(), 8);
    assert_eq!(tess.count_cells(), 8);

    let mut total_vol = 0.0;
    for i in 0..8 {
        let cell = tess.get(i).expect("Should have cell");
        assert_eq!(cell.id(), i);
        let vol: f64 = cell.volume();
        assert!((vol - 125_000.0).abs() < 1e-3, "Cell volume should be 125,000, got {}", vol);
        total_vol += vol;
    }
    assert!((total_vol - 1_000_000.0).abs() < 1e-3, "Total volume should be 1,000,000, got {}", total_vol);
}
