use criterion::{criterion_group, Criterion, BenchmarkId};
use vorothree::{BoundingBox, TessellationGrid, TessellationEdges, TessellationMoctree, Wall};
use vorothree::geometries::TrefoilKnotGeometry;
use rand::prelude::*;
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
}

const SIZES: [usize; 2] = [1_000, 10_000];

fn benchmark_distributions(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut group = c.benchmark_group("distributions");
    group.sample_size(50);
    
    for &size in &SIZES {
        // Grid resolution heuristic: cube root of N
        let grid_res = (size as f64).powf(1.0/3.0).ceil() as usize;
        let grid_res = grid_res.max(1);

        // Uniform Distribution
        group.bench_with_input(BenchmarkId::new("uniform/grid", size), &size, |b, &s| {
            let mut tess = TessellationGrid::new(bounds, grid_res, grid_res, grid_res);
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("uniform/edges", size), &size, |b, &s| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("uniform/moctree", size), &size, |b, &s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        // Trefoil Knot Distribution
        let cx = (bounds.min_x + bounds.max_x) / 2.0;
        let cy = (bounds.min_y + bounds.max_y) / 2.0;
        let cz = (bounds.min_z + bounds.max_z) / 2.0;
        let scale = (bounds.max_x - bounds.min_x).min(bounds.max_y - bounds.min_y).min(bounds.max_z - bounds.min_z) / 7.0;
        let tube_radius = scale * 0.3;

        group.bench_with_input(BenchmarkId::new("trefoil/grid", size), &size, |b, &s| {
            let mut tess = TessellationGrid::new(bounds, grid_res, grid_res, grid_res);
            tess.add_wall(Wall::new(-10, Box::new(TrefoilKnotGeometry::new([cx, cy, cz], scale, tube_radius, 100))));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("trefoil/edges", size), &size, |b, &s| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.add_wall(Wall::new(-10, Box::new(TrefoilKnotGeometry::new([cx, cy, cz], scale, tube_radius, 100))));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("trefoil/moctree", size), &size, |b, &s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
            tess.add_wall(Wall::new(-10, Box::new(TrefoilKnotGeometry::new([cx, cy, cz], scale, tube_radius, 100))));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        // Axes Distribution
        let axes_points = generate_axes_points(size, &bounds);

        group.bench_with_input(BenchmarkId::new("axes/grid", size), &size, |b, &_s| {
            let mut tess = TessellationGrid::new(bounds, grid_res, grid_res, grid_res);
            tess.set_generators(&axes_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("axes/edges", size), &size, |b, &_s| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.set_generators(&axes_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("axes/moctree", size), &size, |b, &_s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
            tess.set_generators(&axes_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        // Central Box Distribution (10% volume)
        let central_points = generate_central_box_points(size, &bounds);

        group.bench_with_input(BenchmarkId::new("central/grid", size), &size, |b, &_s| {
            let mut tess = TessellationGrid::new(bounds, grid_res, grid_res, grid_res);
            tess.set_generators(&central_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("central/edges", size), &size, |b, &_s| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.set_generators(&central_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("central/moctree", size), &size, |b, &_s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
            tess.set_generators(&central_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        // Sphere Surface Distribution
        let sphere_points = generate_sphere_surface_points(size, &bounds);

        group.bench_with_input(BenchmarkId::new("sphere/grid", size), &size, |b, &_s| {
            let mut tess = TessellationGrid::new(bounds, grid_res, grid_res, grid_res);
            tess.set_generators(&sphere_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("sphere/edges", size), &size, |b, &_s| {
            let mut tess = TessellationEdges::new(bounds, grid_res, grid_res, grid_res);
            tess.set_generators(&sphere_points);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("sphere/moctree", size), &size, |b, &_s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
            tess.set_generators(&sphere_points);
            b.iter(|| {
                tess.calculate();
            })
        });
    }
    group.finish();
}

fn generate_axes_points(count: usize, bounds: &BoundingBox) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut points = Vec::with_capacity(count * 3);
    
    let w: f64 = bounds.max_x - bounds.min_x;
    let h: f64 = bounds.max_y - bounds.min_y;
    let d: f64 = bounds.max_z - bounds.min_z;
    
    let transform = |val: f64| 0.5 + 4.0 * (val - 0.5).powi(3);

    for _ in 0..count {
        let x = transform(rng.r#gen::<f64>());
        let y = transform(rng.r#gen::<f64>());
        let z = transform(rng.r#gen::<f64>());

        points.push(bounds.min_x + x * w);
        points.push(bounds.min_y + y * h);
        points.push(bounds.min_z + z * d);
    }
    points
}

fn generate_central_box_points(count: usize, bounds: &BoundingBox) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut points = Vec::with_capacity(count * 3);
    
    let w = bounds.max_x - bounds.min_x;
    let h = bounds.max_y - bounds.min_y;
    let d = bounds.max_z - bounds.min_z;
    
    let cx = (bounds.min_x + bounds.max_x) / 2.0;
    let cy = (bounds.min_y + bounds.max_y) / 2.0;
    let cz = (bounds.min_z + bounds.max_z) / 2.0;

    // Scale factor for 10% volume: s = cbrt(0.1)
    let s = 0.1f64.powf(1.0/3.0);
    
    for _ in 0..count {
        points.push(cx + (rng.r#gen::<f64>() - 0.5) * w * s);
        points.push(cy + (rng.r#gen::<f64>() - 0.5) * h * s);
        points.push(cz + (rng.r#gen::<f64>() - 0.5) * d * s);
    }
    points
}

fn generate_sphere_surface_points(count: usize, bounds: &BoundingBox) -> Vec<f64> {
    let mut rng = rand::thread_rng();
    let mut points = Vec::with_capacity(count * 3);
    
    let w = bounds.max_x - bounds.min_x;
    let h = bounds.max_y - bounds.min_y;
    let d = bounds.max_z - bounds.min_z;
    
    let cx = (bounds.min_x + bounds.max_x) / 2.0;
    let cy = (bounds.min_y + bounds.max_y) / 2.0;
    let cz = (bounds.min_z + bounds.max_z) / 2.0;

    let radius = w.min(h).min(d) / 3.0;

    // Center point
    points.push(cx);
    points.push(cy);
    points.push(cz);
    
    for _ in 1..count {
        loop {
            let x = rng.r#gen::<f64>() * 2.0 - 1.0;
            let y = rng.r#gen::<f64>() * 2.0 - 1.0;
            let z = rng.r#gen::<f64>() * 2.0 - 1.0;
            let len_sq = x*x + y*y + z*z;
            if len_sq > 0.0001 && len_sq <= 1.0 {
                let len = len_sq.sqrt();
                points.push(cx + x / len * radius);
                points.push(cy + y / len * radius);
                points.push(cz + z / len * radius);
                break;
            }
        }
    }
    points
}

fn plot_distribution_results() -> Result<(), Box<dyn std::error::Error>> {
    let distributions = ["uniform", "trefoil", "axes", "central", "sphere"];
    let methods = ["grid", "edges", "moctree"];
    let root = Path::new("target/criterion/distributions");

    if !root.exists() {
        return Ok(());
    }

    let out_dir = Path::new("benches/results");
    std::fs::create_dir_all(out_dir)?;
    
    let output = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output()
        .expect("Failed to execute git command");
    let git_hash = String::from_utf8(output.stdout).expect("Invalid UTF-8").trim().to_string();

    for &size in &SIZES {
        let mut data = BTreeMap::new();
        
        for &dist in &distributions {
            for &method in &methods {
                let id = format!("{}_{}", dist, method);
                let path = root.join(&id).join(size.to_string()).join("base/estimates.json");
                
                if path.exists() {
                    let file = File::open(&path)?;
                    let reader = BufReader::new(file);
                    let estimates: Estimates = serde_json::from_reader(reader)?;
                    data.entry(dist).or_insert_with(BTreeMap::new).insert(method, estimates.mean.point_estimate / 1_000_000.0);
                }
            }
        }

        if data.is_empty() {
            continue;
        }

        let out_file = out_dir.join(format!("bench_distributions_{}k_{}.png", size / 1000, git_hash));
        let root_area = BitMapBackend::new(&out_file, (1024, 768)).into_drawing_area();
        root_area.fill(&WHITE)?;

        let max_y = data.values()
            .flat_map(|m| m.values())
            .fold(0.0f64, |a, &b| a.max(b));

        let mut chart = ChartBuilder::on(&root_area)
            .caption(format!("Distribution Benchmark (N={})", size), ("sans-serif", 40).into_font())
            .margin(20)
            .x_label_area_size(40)
            .y_label_area_size(80)
            .build_cartesian_2d(
                -0.5..(distributions.len() as f64 - 0.5),
                0.0..max_y * 1.1
            )?;

        chart.configure_mesh()
            .x_labels(distributions.len())
            .x_label_formatter(&|v| {
                let idx = v.round() as isize;
                if idx >= 0 && (idx as usize) < distributions.len() && (v - idx as f64).abs() < 0.1 {
                    distributions[idx as usize].to_string()
                } else {
                    "".to_string()
                }
            })
            .y_desc("Time (ms)")
            .draw()?;

        let colors = [RED, GREEN, BLUE];

        for (i, method) in methods.iter().enumerate() {
            let color = colors[i % colors.len()];
            chart.draw_series(
                data.iter().map(|(dist, method_map)| {
                    let dist_idx = distributions.iter().position(|d| d == dist).unwrap();
                    let val = *method_map.get(*method).unwrap_or(&0.0);
                    
                    let group_center = dist_idx as f64;
                    let offset = if i == 0 { -0.2 } else { 0.2 };
                    let bar_center = group_center + offset;
                    let width = 0.35;
                    
                    Rectangle::new(
                        [(bar_center - width / 2.0, 0.0), (bar_center + width / 2.0, val)],
                        color.filled()
                    )
                })
            )?
            .label(*method)
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], color));
        }
        
        chart.configure_series_labels()
            .background_style(&WHITE.mix(0.8))
            .border_style(&BLACK)
            .draw()?;
            
        println!("Plot saved to {:?}", out_file);
    }
    Ok(())
}

criterion_group!(benches, benchmark_distributions);

fn main() {
    benches();
    if let Err(e) = plot_distribution_results() {
        eprintln!("Error generating plot: {}", e);
    }
}