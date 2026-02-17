use criterion::{criterion_group, Criterion, BenchmarkId};
use vorothree::{BoundingBox, Tessellation, TessellationEdges, TessellationMoctree};
use plotters::prelude::*;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Deserialize)]
struct Estimates {
    mean: Stats,
}

#[derive(Deserialize)]
struct Stats {
    point_estimate: f64,
}

fn benchmark_scaling(c: &mut Criterion) {
    let sizes = [10, 100, 1000, 10_000, 100_000, 1_000_000];
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut group = c.benchmark_group("scaling");
    group.sample_size(50);
    
    for &size in &sizes {
        // Grid resolution heuristic: cube root of N
        let grid_res = (size as f64).powf(1.0/3.0).ceil() as usize;
        let grid_res = grid_res.max(1);
        
        let total_cells = grid_res * grid_res * grid_res;
        let density = size as f64 / total_cells as f64;
        println!("N: {:7}, Grid: {:3}x{:3}x{:3}, Cells: {:9}, Density: {:.3}", size, grid_res, grid_res, grid_res, total_cells, density);

        group.bench_with_input(BenchmarkId::new("grid", size), &size, |b, &s| {
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("edges", size), &size, |b, &s| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("moctree", size), &size, |b, &s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
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
    let sizes = [10, 100, 1000, 10_000, 100_000, 1_000_000];
    let root = Path::new("target/criterion/scaling");

    if !root.exists() {
        return Ok(());
    }

    let mut data: BTreeMap<&str, Vec<(u32, f64)>> = BTreeMap::new();

    for &method in &methods {
        let mut points = Vec::new();
        for &size in &sizes {
            let path = root
                .join(method)
                .join(size.to_string())
                .join("base/estimates.json");

            if path.exists() {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                let estimates: Estimates = serde_json::from_reader(reader)?;
                points.push((size, estimates.mean.point_estimate));
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
    let out_file = out_dir.join("scaling_benchmark.png");
    let root_area = BitMapBackend::new(&out_file, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let min_y = data.values().flat_map(|v| v.iter().map(|p| p.1)).fold(f64::INFINITY, f64::min);
    let max_y = data.values().flat_map(|v| v.iter().map(|p| p.1)).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root_area)
        .caption("Scaling Benchmark Results", ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (sizes[0] as f64..*sizes.last().unwrap() as f64).log_scale(),
            (min_y * 0.8..max_y * 1.5).log_scale(),
        )?;

    chart.configure_mesh()
        .x_desc("Number of Points (N)")
        .y_desc("Time (ns)")
        .draw()?;

    let colors = [RED, BLUE, GREEN, MAGENTA, CYAN];

    for (i, (method, points)) in data.iter().enumerate() {
        let color = colors[i % colors.len()];

        chart
            .draw_series(LineSeries::new(
                points.iter().map(|(x, y)| (*x as f64, *y)),
                &color,
            ))?
            .label(*method)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));

        chart.draw_series(PointSeries::of_element(
            points.iter().map(|(x, y)| (*x as f64, *y)),
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