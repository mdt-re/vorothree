use vorothree::{BoundingBox, Tessellation, AlgorithmGrid, CellFaces};

#[test]
fn test_mapping_face_counts() {
    let bounds = BoundingBox::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);
    // Use a grid algorithm for spatial indexing
    let algo = AlgorithmGrid::new(4, 4, 4, &bounds);
    let mut tess = Tessellation::<3, CellFaces, _>::new(bounds, algo);

    // Create a 2x2x2 grid of points (8 generators)
    // These points are perfectly symmetric in the 1x1x1 box.
    let mut generators = Vec::new();
    for x in [0.25, 0.75] {
        for y in [0.25, 0.75] {
            for z in [0.25, 0.75] {
                generators.push(x);
                generators.push(y);
                generators.push(z);
            }
        }
    }
    tess.set_generators(&generators);

    // Use map_cells to calculate the number of faces for each cell.
    // This avoids storing the full Cell objects in memory.
    let face_counts: Vec<usize> = tess.map(|cell| cell.face_counts().len());

    assert_eq!(face_counts.len(), 8, "Should have results for 8 cells");
    
    // In a 2x2x2 grid inside a box, each cell is a smaller rectangular prism (cube in this case).
    // Each cell touches 3 walls of the bounding box and 3 other cells.
    // Therefore, every cell should have exactly 6 faces.
    for (i, &count) in face_counts.iter().enumerate() {
        assert_eq!(count, 6, "Cell {} should have 6 faces", i);
    }
}