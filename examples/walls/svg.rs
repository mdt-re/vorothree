use plotters::prelude::*;
use rand::Rng;
use vorothree::{BoundingBox, Tessellation, Wall};
use vorothree::geometries::{TrefoilKnotGeometry, PlaneGeometry, SphereGeometry, CylinderGeometry, TorusGeometry};

fn draw_tessellation(
    tess: &Tessellation,
    generators: &[f64],
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let root = SVGBackend::new(filename, (1024, 768)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("3D Voronoi - {}", filename), ("sans-serif", 20))
        .margin(20)
        .x_label_area_size(30)
        .y_label_area_size(30)
        .build_cartesian_3d(0.0..100.0, 0.0..100.0, 0.0..100.0)?;

    chart.configure_axes().draw()?;

    // Draw the cells with transparency
    for i in 0..tess.count_cells() {
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
    println!("Output saved to {}", filename);
    Ok(())
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Setup the tessellation
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);

    // Generate random points
    let mut rng = rand::thread_rng();
    let mut generators = Vec::new();
    for _ in 0..1000 {
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
        generators.push(rng.gen_range(0.0..100.0));
    }

    // Run 1: Plane Wall
    {
        let mut tess = Tessellation::new(bounds.clone(), 10, 10, 10);
        tess.set_generators(&generators);

        tess.add_wall(Wall::new(
            -10,
            Box::new(PlaneGeometry::new([40.0, 40.0, 40.0], [1.0, 1.0, 1.0]))
        ));
        tess.calculate();
        draw_tessellation(&tess, &generators, "wall_plane.svg")?;
    }

    // Run 2: Sphere Wall
    {
        let mut tess = Tessellation::new(bounds.clone(), 10, 10, 10);
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-11, Box::new(SphereGeometry::new([50.0, 50.0, 50.0], 40.0))));
        tess.calculate();
        draw_tessellation(&tess, &generators, "wall_sphere.svg")?;
    }
    
    // Run 3: Cylinder Wall
    {
        let mut tess = Tessellation::new(bounds.clone(), 10, 10, 10);
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-12, Box::new(CylinderGeometry::new([50.0, 50.0, 50.0], [0.0, 0.0, 1.0], 40.0))));
        tess.calculate();
        draw_tessellation(&tess, &generators, "wall_cylinder.svg")?;
    }

    // Run 4: Torus Wall
    {
        let mut tess = Tessellation::new(bounds.clone(), 10, 10, 10);
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-13, Box::new(TorusGeometry::new([50.0, 50.0, 50.0], [0.0, 0.0, 1.0], 35.0, 10.0))));
        tess.calculate();
        draw_tessellation(&tess, &generators, "wall_torus.svg")?;
    }

    // Run 6: Trefoil Knot Wall (Custom)
    {
        let mut tess = Tessellation::new(bounds.clone(), 10, 10, 10);
        tess.set_generators(&generators);
        tess.add_wall(Wall::new(-15, Box::new(TrefoilKnotGeometry::new(
            [50.0, 50.0, 50.0],
            12.0,
            8.0,
            200
        ))));
        tess.calculate();
        draw_tessellation(&tess, &generators, "wall_knot.svg")?;
    }

    Ok(())
}