use plotters::prelude::*;
use rand::Rng;
use vorothree::{BoundingBox, Tessellation};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Define the output file and dimensions
    let filename = "tessellation.svg";
    let root = SVGBackend::new(filename, (1024, 768)).into_drawing_area();
    
    root.fill(&WHITE)?;

    // Create a 3D chart context
    let mut chart = ChartBuilder::on(&root)
        .caption("3D Voronoi Generators", ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_3d(0.0..100.0, 0.0..100.0, 0.0..100.0)?;

    chart.configure_axes().draw()?;

    // Setup the tessellation
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    let mut tess = Tessellation::new(bounds, 10, 10, 10);

    // Generate random points
    let mut rng = rand::thread_rng();
    let mut generators = Vec::new();
    for _ in 0..100 {
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
    }

    tess.set_generators(&generators);
    tess.calculate();

    // Draw the cells with transparency
    // Note: This assumes the Cell API exposes faces and vertices.
    for i in 0..100 {
        if let Some(cell) = tess.get(i) {
            let vertices = cell.vertices();
            for face in cell.faces() {
                let poly: Vec<(f64, f64, f64)> = face
                    .iter()
                    .map(|&idx| (vertices[idx * 3], vertices[idx * 3 + 1], vertices[idx * 3 + 2]))
                    .collect();
                chart.draw_series(std::iter::once(Polygon::new(poly, BLUE.mix(0.1).filled())))?;
            }
        }
    }

    // Draw the generators as points
    let points: Vec<(f64, f64, f64)> = generators
        .chunks(3)
        .map(|c| (c[0], c[1], c[2]))
        .collect();

    chart.draw_series(points.iter().map(|&p| Circle::new(p, 3, RED.filled())))?;

    root.present()?;
    println!("Example output saved to {}", filename);
    Ok(())
}