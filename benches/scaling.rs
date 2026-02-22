use criterion::{criterion_group, Criterion, BenchmarkId};
use vorothree::{BoundingBox, Tessellation, AlgorithmGrid, AlgorithmOctree, CellFaces, CellEdges};
use plotters::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::process::Command;

#[derive(Deserialize)]
struct Estimates {
    mean: Stats,
}

#[derive(Deserialize)]
struct Stats {
    point_estimate: f64,
    confidence_interval: ConfidenceInterval,
}

#[derive(Deserialize)]
struct ConfidenceInterval {
    lower_bound: f64,
    upper_bound: f64,
}

//const SIZES: [usize; 7] = [10, 100, 1000, 10_000, 100_000, 1_000_000, 10_000_000];
const SIZES: [usize; 5] = [10, 100, 1000, 10_000, 100_000];

fn benchmark_scaling(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut group = c.benchmark_group("scaling");
    group.sample_size(10);
    
    for &size in &SIZES {
        // Grid resolution heuristic: cube root of N
        let grid_res = (size as f64).powf(1.0/3.0).ceil() as usize;
        let grid_res = grid_res.max(1);
        
        let total_cells = grid_res * grid_res * grid_res;
        let density = size as f64 / total_cells as f64;
        println!("N: {:7}, Grid: {:3}x{:3}x{:3}, Cells: {:9}, Density: {:.3}", size, grid_res, grid_res, grid_res, total_cells, density);

        group.bench_with_input(BenchmarkId::new("grid", size), &size, |b, &s| {
            let mut tess = Tessellation::<CellFaces, _>::new(bounds, AlgorithmGrid::new(grid_res, grid_res, grid_res, &bounds));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("edges", size), &size, |b, &s| {
            let mut tess = Tessellation::<CellEdges, _>::new(bounds, AlgorithmGrid::new(grid_res, grid_res, grid_res, &bounds));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("moctree", size), &size, |b, &s| {
            let mut tess = Tessellation::<CellFaces, _>::new(bounds, AlgorithmOctree::new(bounds, 8));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });
    }
    group.finish();
}

fn plot_scaling_results() -> Result<(), Box<dyn std::error::Error>> {
    let methods = ["grid", "edges", "moctree"];
    let root = Path::new("target/criterion/scaling");

    if !root.exists() {
        return Ok(());
    }

    let mut data: BTreeMap<&str, Vec<(usize, f64, f64, f64)>> = BTreeMap::new();

    for &method in &methods {
        let mut points = Vec::new();
        for &size in &SIZES {
            let path = root
                .join(method)
                .join(size.to_string())
                .join("base/estimates.json");

            if path.exists() {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                let estimates: Estimates = serde_json::from_reader(reader)?;
                points.push((
                    size,
                    estimates.mean.point_estimate / 1_000_000.0,
                    estimates.mean.confidence_interval.lower_bound / 1_000_000.0,
                    estimates.mean.confidence_interval.upper_bound / 1_000_000.0,
                ));
            }
        }
        if !points.is_empty() {
            points.sort_by_key(|k| k.0);
            data.insert(method, points);
        }
    }

    if data.is_empty() {
        return Ok(());
    }

    let out_dir = Path::new("benches/results");
    std::fs::create_dir_all(out_dir)?;
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");
    let git_hash = String::from_utf8(output.stdout).expect("Invalid UTF-8").trim().to_string();
    let out_file = out_dir.join(format!("bench_scaling_{}.png", git_hash));
    let root_area = BitMapBackend::new(&out_file, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let min_y = data.values().flat_map(|v| v.iter().map(|p| p.2)).fold(f64::INFINITY, f64::min);
    let max_y = data.values().flat_map(|v| v.iter().map(|p| p.3)).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Scaling Benchmark Results", ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (SIZES[0] as f64..*SIZES.last().unwrap() as f64).log_scale(),
            (min_y * 0.8..max_y * 1.5).log_scale(),
        )?;

    chart.configure_mesh()
        .x_desc("Number of Points (N)")
        .y_desc("Time (ms)")
        .draw()?;

    // Draw Linear and Quadratic Scaling References (Dotted Lines)
    if let Some(first_series) = data.values().next() {
        if let Some(&(start_n, start_t, _, _)) = first_series.first() {
            let start_n = start_n as f64;
            let end_n = *SIZES.last().unwrap() as f64;
            
            // Logarithmic steps for uniform dots on log-scale
            let step = 10.0f64.powf(0.05); 

            // Linear: y = x * (start_t / start_n)
            let mut linear_points = Vec::new();
            let mut n = SIZES[0] as f64;
            while n <= end_n * 1.1 {
                let t = start_t * (n / start_n);
                linear_points.push((n, t));
                n *= step;
            }

            chart.draw_series(PointSeries::of_element(
                linear_points,
                1,
                &BLACK,
                &|c, s, st| Circle::new(c, s, st.filled()),
            ))?
            .label("Linear")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));

            // Quadratic: y = x^2 * (start_t / start_n^2)
            let mut quadratic_points = Vec::new();
            let mut n = SIZES[0] as f64;
            while n <= end_n * 1.1 {
                let t = start_t * (n / start_n).powi(2);
                quadratic_points.push((n, t));
                n *= step;
            }

            chart.draw_series(PointSeries::of_element(
                quadratic_points,
                1,
                &BLACK,
                &|c, s, st| Circle::new(c, s, st.filled()),
            ))?
            .label("Quadratic")
            .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &BLACK));
        }
    }

    let colors = [RED, BLUE, GREEN, MAGENTA, CYAN];

    for (i, (method, points)) in data.iter().enumerate() {
        let color = colors[i % colors.len()];

        let mut band_points = Vec::new();
        for (x, _, _, u) in points.iter() {
            band_points.push((*x as f64, *u));
        }
        for (x, _, l, _) in points.iter().rev() {
            band_points.push((*x as f64, *l));
        }

        chart.draw_series(std::iter::once(Polygon::new(
            band_points,
            color.mix(0.2).filled(),
        )))?;

        chart
            .draw_series(LineSeries::new(
                points.iter().map(|(x, y, _, _)| (*x as f64, *y)),
                &color,
            ))?
            .label(*method)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));

        chart.draw_series(PointSeries::of_element(
            points.iter().map(|(x, y, _, _)| (*x as f64, *y)),
            5,
            &color,
            &|c, s, st| {
                return EmptyElement::at(c) + Circle::new((0, 0), s, st.filled());
            },
        ))?;
    }

    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    println!("Plot saved to {:?}", out_file);

    Ok(())
}

criterion_group!(benches, benchmark_scaling);

fn main() {
    benches();
    if let Err(e) = plot_scaling_results() {
        eprintln!("Error generating plot: {}", e);
    }
}