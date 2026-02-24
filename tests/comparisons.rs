use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use vorothree::geometries::SphereGeometry;
use vorothree::{AlgorithmGrid, BoundingBox, CellFaces, Tessellation, Wall, WALL_ID_START};

#[test]
fn test_comparisons_face_counts() {
    // Locate the data directory relative to the manifest
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let data_dir = PathBuf::from(manifest_dir).join("tests").join("data").join("face_counts");

    let input_path = data_dir.join("generators.txt");
    let output_path = data_dir.join("output.txt");

    // Define bounds that cover the input data (approx -10 to 10 based on the file content)
    let bounds = BoundingBox::new(-10.0, -10.0, -10.0, 10.0, 10.0, 10.0);

    // Create tessellation with grid algorithm
    // 10x10x10 grid is a reasonable default for this volume
    let algo = AlgorithmGrid::new(6, 6, 6, &bounds);
    let mut tess = Tessellation::<CellFaces, _>::new(bounds, algo);

    // Add a spherical wall with radius 8
    tess.add_wall(Wall::new(
        WALL_ID_START,
        Box::new(SphereGeometry::new([0.0, 0.0, 0.0], 8.0)),
    ));

    // Import generators from the input file
    tess.import_generators(&input_path)
        .expect("Failed to import generators");

    // Calculate the tessellation
    tess.calculate();

    // Read expected vertex positions from output file
    let file = File::open(&output_path).expect("Failed to open output.txt");
    let reader = BufReader::new(file);

    let mut expected_cells: HashMap<usize, Vec<[f64; 3]>> = HashMap::new();
    let mut current_id = None;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("Cell") {
            // Format: "Cell <id>:"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id_str = parts[1].trim_end_matches(':');
                if let Ok(id) = id_str.parse::<usize>() {
                    current_id = Some(id);
                    expected_cells.insert(id, Vec::new());
                }
            }
        } else if let Some(id) = current_id {
            // Format: "x y z"
            let coords: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            if coords.len() == 3 {
                expected_cells
                    .get_mut(&id)
                    .unwrap()
                    .push([coords[0], coords[1], coords[2]]);
            }
        }
    }

    // Verify results
    for (id, expected) in expected_cells {
        let cell = tess.get_cell(id).expect("Cell not found");
        let vertices = cell.vertices();
        let calculated: Vec<[f64; 3]> = vertices
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();

        assert_eq!(
            calculated.len(),
            expected.len(),
            "Vertex count mismatch for cell {}",
            id
        );

        // Check if every expected vertex exists in the calculated set (order may differ)
        for exp_v in &expected {
            let found = calculated.iter().any(|calc_v| {
                (calc_v[0] - exp_v[0]).abs() < 1e-4
                    && (calc_v[1] - exp_v[1]).abs() < 1e-4
                    && (calc_v[2] - exp_v[2]).abs() < 1e-4
            });
            assert!(
                found,
                "Vertex {:?} not found in cell {} (found {:?})",
                exp_v, id, calculated
            );
        }
    }
}

#[test]
fn test_comparisons_vertex_positions() {
    // Locate the data directory relative to the manifest
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let data_dir = PathBuf::from(manifest_dir).join("tests").join("data").join("vertex_positions");

    let input_path = data_dir.join("generators.txt");
    let output_path = data_dir.join("output.txt");

    // Define bounds that cover the input data (approx -10 to 10 based on the file content)
    let bounds = BoundingBox::new(-10.0, -10.0, -10.0, 10.0, 10.0, 10.0);

    // Create tessellation with grid algorithm
    // 10x10x10 grid is a reasonable default for this volume
    let algo = AlgorithmGrid::new(6, 6, 6, &bounds);
    let mut tess = Tessellation::<CellFaces, _>::new(bounds, algo);

    // Add a spherical wall with radius 8
    tess.add_wall(Wall::new(
        WALL_ID_START,
        Box::new(SphereGeometry::new([0.0, 0.0, 0.0], 8.0)),
    ));

    // Import generators from the input file
    tess.import_generators(&input_path)
        .expect("Failed to import generators");

    // Calculate the tessellation
    tess.calculate();

    // Read expected vertex positions from output file
    let file = File::open(&output_path).expect("Failed to open output.txt");
    let reader = BufReader::new(file);

    let mut expected_cells: HashMap<usize, Vec<[f64; 3]>> = HashMap::new();
    let mut current_id = None;

    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("Cell") {
            // Format: "Cell <id>:"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let id_str = parts[1].trim_end_matches(':');
                if let Ok(id) = id_str.parse::<usize>() {
                    current_id = Some(id);
                    expected_cells.insert(id, Vec::new());
                }
            }
        } else if let Some(id) = current_id {
            // Format: "x y z"
            let coords: Vec<f64> = line
                .split_whitespace()
                .filter_map(|s| s.parse::<f64>().ok())
                .collect();
            if coords.len() == 3 {
                expected_cells
                    .get_mut(&id)
                    .unwrap()
                    .push([coords[0], coords[1], coords[2]]);
            }
        }
    }

    // Verify results
    for (id, expected) in expected_cells {
        let cell = tess.get_cell(id).expect("Cell not found");
        let vertices = cell.vertices();
        let calculated: Vec<[f64; 3]> = vertices
            .chunks(3)
            .map(|c| [c[0], c[1], c[2]])
            .collect();

        assert_eq!(
            calculated.len(),
            expected.len(),
            "Vertex count mismatch for cell {}",
            id
        );

        // Check if every expected vertex exists in the calculated set (order may differ)
        for exp_v in &expected {
            let found = calculated.iter().any(|calc_v| {
                (calc_v[0] - exp_v[0]).abs() < 1e-4
                    && (calc_v[1] - exp_v[1]).abs() < 1e-4
                    && (calc_v[2] - exp_v[2]).abs() < 1e-4
            });
            assert!(
                found,
                "Vertex {:?} not found in cell {} (found {:?})",
                exp_v, id, calculated
            );
        }
    }
}