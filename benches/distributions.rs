use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use vorothree::{BoundingBox, Tessellation, TessellationMoctree, Wall};
use vorothree::geometries::TrefoilKnotGeometry;
use rand::prelude::*;

fn benchmark_distributions(c: &mut Criterion) {
    let sizes = [1_000, 10_000];
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut group = c.benchmark_group("distributions");
    group.sample_size(50);
    
    for &size in &sizes {
        // Grid resolution heuristic: cube root of N
        let grid_res = (size as f64).powf(1.0/3.0).ceil() as usize;
        let grid_res = grid_res.max(1);

        // Uniform Distribution
        group.bench_with_input(BenchmarkId::new("uniform/grid", size), &size, |b, &s| {
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
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
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
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
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
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
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
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
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
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

criterion_group!(benches, benchmark_distributions);
criterion_main!(benches);