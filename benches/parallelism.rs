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

const N_POINTS: usize = 1_000_000;

fn benchmark_parallelism(c: &mut Criterion) {
    let bounds = BoundingBox::new([0.0, 0.0, 0.0], [100.0, 100.0, 100.0]);
    
    let mut group = c.benchmark_group(format!("parallelism_{}k", N_POINTS / 1000));
    group.sample_size(10);

    let max_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
    let mut cores_list = Vec::new();
    let mut cores = 1;
    while cores <= max_cores {
        cores_list.push(cores);
        cores *= 2;
    }
    if cores_list.last().map_or(false, |&last| last < max_cores) {
        cores_list.push(max_cores);
    }
    
    // Grid resolution heuristic: cube root of N
    let grid_res = (N_POINTS as f64).powf(1.0/3.0).ceil() as usize;
    let grid_res = grid_res.max(1);

    for &num_threads in &cores_list {
        // Create a thread pool for this specific number of threads
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()
            .unwrap();

        group.bench_with_input(BenchmarkId::new("grid", num_threads), &num_threads, |b, &_s| {
            let mut tess = Tessellation::<3, CellFaces, _>::new(bounds, AlgorithmGrid::new(grid_res, grid_res, grid_res, &bounds));
            tess.random_generators(N_POINTS);
            b.iter(|| {
                pool.install(|| {
                    tess.calculate();
                })
            })
        });

        group.bench_with_input(BenchmarkId::new("edges", num_threads), &num_threads, |b, &_s| {
            let mut tess = Tessellation::<3, CellEdges, _>::new(bounds, AlgorithmGrid::new(grid_res, grid_res, grid_res, &bounds));
            tess.random_generators(N_POINTS);
            b.iter(|| {
                pool.install(|| {
                    tess.calculate();
                })
            })
        });

        group.bench_with_input(BenchmarkId::new("moctree", num_threads), &num_threads, |b, &_s| {
            let mut tess = Tessellation::<3, CellFaces, _>::new(bounds, AlgorithmOctree::new(bounds, 8));
            tess.random_generators(N_POINTS);
            b.iter(|| {
                pool.install(|| {
                    tess.calculate();
                })
            })
        });
    }
    group.finish();
}

fn plot_parallelism_results() -> Result<(), Box<dyn std::error::Error>> {
    let methods = ["grid", "edges", "moctree"];
    let root_dir = format!("target/criterion/parallelism_{}k", N_POINTS / 1000);
    let root = Path::new(&root_dir);

    if !root.exists() {
        return Ok(());
    }

    let mut data: BTreeMap<&str, Vec<(usize, f64, f64, f64)>> = BTreeMap::new();
    
    let max_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(8);
    let mut cores_list = Vec::new();
    let mut cores = 1;
    while cores <= max_cores {
        cores_list.push(cores);
        cores *= 2;
    }
    if cores_list.last().map_or(false, |&last| last < max_cores) {
        cores_list.push(max_cores);
    }

    for &method in &methods {
        let mut points = Vec::new();
        for &num_threads in &cores_list {
            let path = root
                .join(method)
                .join(num_threads.to_string())
                .join("base/estimates.json");

            if path.exists() {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                let estimates: Estimates = serde_json::from_reader(reader)?;
                points.push((
                    num_threads,
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
        .ok();
    let git_hash = output
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();
        
    let out_file = out_dir.join(format!("bench_parallelism_{}k_{}.png", N_POINTS / 1000, git_hash));
    let root_area = BitMapBackend::new(&out_file, (1024, 768)).into_drawing_area();
    root_area.fill(&WHITE)?;

    let max_y = data.values().flat_map(|v| v.iter().map(|p| p.3)).fold(f64::NEG_INFINITY, f64::max);
    
    let min_x = cores_list[0];
    let max_x = *cores_list.last().unwrap();

    let mut chart = ChartBuilder::on(&root_area)
        .caption(format!("Parallelism Benchmark (N={})", N_POINTS), ("sans-serif", 40).into_font())
        .margin(20)
        .x_label_area_size(40)
        .y_label_area_size(80)
        .build_cartesian_2d(
            min_x as f64..max_x as f64,
            0.0..max_y * 1.1,
        )?;

    chart.configure_mesh()
        .x_desc("Number of Cores")
        .y_desc("Time (ms)")
        .x_labels(cores_list.len())
        .x_label_formatter(&|v| format!("{:.0}", v))
        .draw()?;

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

criterion_group!(benches, benchmark_parallelism);

fn main() {
    benches();
    if let Err(e) = plot_parallelism_results() {
        eprintln!("Error generating plot: {}", e);
    }
}