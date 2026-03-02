use plotters::prelude::*;
use rand::Rng;
use vorothree::{AlgorithmGrid2D, BoundingBox, Cell2D, Tessellation, Wall};
use vorothree::geometries::{LineGeometry, CircleGeometry, ConvexPolygonGeometry2D, AnnulusGeometry, CubicBezierGeometry2D};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    run_example("2d_tessellation_line.svg", |tess| {
        tess.add_wall(Wall::new(-1000, Box::new(LineGeometry::new([50.0, 50.0], [1.0, 1.0]))));
    })?;

    run_example("2d_tessellation_circle.svg", |tess| {
        tess.add_wall(Wall::new(-1000, Box::new(CircleGeometry::new([50.0, 50.0], 40.0))));
    })?;

    run_example("2d_tessellation_polygon.svg", |tess| {
        tess.add_wall(Wall::new(-1000, Box::new(ConvexPolygonGeometry2D::new_regular([50.0, 50.0], 40.0, 6))));
    })?;

    run_example("2d_tessellation_washer.svg", |tess| {
        tess.add_wall(Wall::new(-1000, Box::new(AnnulusGeometry::new([50.0, 50.0], 15.0, 45.0))));
    })?;

    run_example("2d_tessellation_spline.svg", |tess| {
        tess.add_wall(Wall::new(-1000, Box::new(CubicBezierGeometry2D::new(
            [10.0, 50.0], [10.0, 10.0], [90.0, 90.0], [90.0, 50.0], 
            15.0, 20, false
        ))));
    })?;

    Ok(())
}

fn run_example<F>(filename: &str, setup_walls: F) -> Result<(), Box<dyn std::error::Error>> 
where F: Fn(&mut Tessellation<2, Cell2D, AlgorithmGrid2D>) {
    let root = SVGBackend::new(filename, (1024, 1024)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .build_cartesian_2d(0.0..100.0, 0.0..100.0)?;

    let bounds = BoundingBox::new([0.0, 0.0], [100.0, 100.0]);
    let algo = AlgorithmGrid2D::new(30, 30, &bounds);
    let mut tess = Tessellation::<2, Cell2D, _>::new(bounds, algo);

    let mut rng = rand::thread_rng();
    let mut generators = Vec::with_capacity(1000 * 2);
    for _ in 0..1000 {
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
    }
    tess.set_generators(&generators);
    
    setup_walls(&mut tess);
    
    tess.calculate();

    // Draw bounding box
    chart.draw_series(std::iter::once(PathElement::new(
        vec![(0.0, 0.0), (100.0, 0.0), (100.0, 100.0), (0.0, 100.0), (0.0, 0.0)],
        BLACK.stroke_width(2),
    )))?;

    // Draw cells
    for i in 0..tess.count_cells() {
        if let Some(cell) = tess.get_cell(i) {
            let vertices = cell.vertices();
            if vertices.len() < 6 {
                continue;
            }

            let mut poly = Vec::new();
            for j in 0..(vertices.len() / 2) {
                poly.push((vertices[j * 2], vertices[j * 2 + 1]));
            }

            chart.draw_series(std::iter::once(Polygon::new(
                poly.clone(),
                BLUE.mix(0.1).filled(),
            )))?;

            poly.push(poly[0]);
            chart.draw_series(std::iter::once(PathElement::new(
                poly,
                BLACK.mix(0.5),
            )))?;
        }
    }

    // Draw generators
    let points: Vec<(f64, f64)> = generators.chunks(2).map(|c| (c[0], c[1])).collect();
    chart.draw_series(points.iter().map(|&p| Circle::new(p, 2, RED.filled())))?;

    root.present()?;
    println!("Output saved to {}", filename);
    Ok(())
}