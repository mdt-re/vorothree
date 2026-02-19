use criterion::{criterion_group, Criterion, BenchmarkId};
use vorothree::{BoundingBox, Tessellation, TessellationEdges, TessellationMoctree};
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

// Test for a given number of points and a range of capacities (generators per bin/leaf)
const N_POINTS: usize = 100_000;
const CAPACITIES: [f64; 9] = [0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 20.0];


fn benchmark_capacity(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);

    let mut group = c.benchmark_group(format!("capacity_{}k", N_POINTS / 1000));
    group.sample_size(20);
    
    for &cap in &CAPACITIES {
        // For grids: resolution derived from desired density (capacity)
        let total_cells = N_POINTS as f64 / cap;
        let grid_res = total_cells.powf(1.0/3.0).ceil() as usize;
        let grid_res = grid_res.max(1);
        
        // For Moctree: capacity is the bucket size. 
        let moctree_cap = (cap.ceil() as usize).max(1);

        println!("Cap: {:.1}, Grid Res: {}, Moctree Cap: {}", cap, grid_res, moctree_cap);

        group.bench_with_input(BenchmarkId::new("grid", format!("{:.1}", cap)), &cap, |b, &_| {
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
            tess.random_generators(N_POINTS);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("edges", format!("{:.1}", cap)), &cap, |b, &_| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.random_generators(N_POINTS);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("moctree", format!("{:.1}", cap)), &cap, |b, &_| {
            let mut tess = TessellationMoctree::new(bounds, moctree_cap);
            tess.random_generators(N_POINTS);
            b.iter(|| {
                tess.calculate();
            })
        });
    }
    group.finish();
}

fn plot_capacity_results() -> Result<(), Box<dyn std::error::Error>> {
    let methods = ["grid", "edges", "moctree"];
    let root_dir = format!("target/criterion/capacity_{}k", N_POINTS / 1000);
    let root = Path::new(&root_dir);

    if !root.exists() {
        return Ok(());
    }

    let mut data: BTreeMap<&str, Vec<(f64, f64, f64, f64)>> = BTreeMap::new();

    for &method in &methods {
        let mut points = Vec::new();
        for &cap in &CAPACITIES {
            let path = root
                .join(method)
                .join(format!("{:.1}", cap))
                .join("base/estimates.json");

            if path.exists() {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                let estimates: Estimates = serde_json::from_reader(reader)?;
                points.push((
                    cap,
                    estimates.mean.point_estimate,
                    estimates.mean.confidence_interval.lower_bound,
                    estimates.mean.confidence_interval.upper_bound,
                ));
            }
        }
        if !points.is_empty() {
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
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
    let out_file = out_dir.join(format!("bench_capacity_{}k_{}.png", N_POINTS / 1000, git_hash));
    let root_area = BitMapBackend::new(&out_file, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let min_y = data.values().flat_map(|v| v.iter().map(|p| p.2)).fold(f64::INFINITY, f64::min);
    let max_y = data.values().flat_map(|v| v.iter().map(|p| p.3)).fold(f64::NEG_INFINITY, f64::max);

    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("Capacity Benchmark Results (N={})", N_POINTS), ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (CAPACITIES[0]..CAPACITIES[CAPACITIES.len()-1]).log_scale(),
            (min_y * 0.8..max_y * 1.5).log_scale(),
        )?;

    chart.configure_mesh()
        .x_desc("Capacity (Points per Bin/Leaf)")
        .y_desc("Time (ns)")
        .draw()?;

    let colors = [RED, BLUE, GREEN, MAGENTA, CYAN];

    for (i, (method, points)) in data.iter().enumerate() {
        let color = colors[i % colors.len()];

        let mut band_points = Vec::new();
        for (x, _, _, u) in points.iter() {
            band_points.push((*x, *u));
        }
        for (x, _, l, _) in points.iter().rev() {
            band_points.push((*x, *l));
        }

        chart.draw_series(std::iter::once(Polygon::new(
            band_points,
            color.mix(0.2).filled(),
        )))?;

        chart
            .draw_series(LineSeries::new(
                points.iter().map(|(x, y, _, _)| (*x, *y)),
                &color,
            ))?
            .label(*method)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], &color));

        chart.draw_series(PointSeries::of_element(
            points.iter().map(|(x, y, _, _)| (*x, *y)),
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

criterion_group!(benches, benchmark_capacity);

fn main() {
    benches();
    if let Err(e) = plot_capacity_results() {
        eprintln!("Error generating plot: {}", e);
    }
}