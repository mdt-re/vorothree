use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use vorothree::{BoundingBox, Tessellation, TessellationMoctree, TessellationKdTree};

fn benchmark_scaling(c: &mut Criterion) {
    let sizes = [10, 100, 1000, 10000, 100000];
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut group = c.benchmark_group("scaling");
    group.sample_size(10);
    
    for &size in &sizes {
        // Grid resolution heuristic: cube root of N
        let grid_res = (size as f64).powf(1.0/3.0).ceil() as usize;
        let grid_res = grid_res.max(1);

        group.bench_with_input(BenchmarkId::new("grid", size), &size, |b, &s| {
            let mut tess = Tessellation::new(bounds, grid_res, grid_res, grid_res);
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

        group.bench_with_input(BenchmarkId::new("kdtree", size), &size, |b, &s| {
            let mut tess = TessellationKdTree::new(bounds);
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_scaling);
criterion_main!(benches);