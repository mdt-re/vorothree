use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use vorothree::{BoundingBox, Tessellation, TessellationMoctree, TessellationKdTree, Wall};

fn benchmark_distributions(c: &mut Criterion) {
    let sizes = [10_000, 100_000];
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut group = c.benchmark_group("distributions");
    group.sample_size(10);
    
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

        group.bench_with_input(BenchmarkId::new("uniform/kdtree", size), &size, |b, &s| {
            let mut tess = TessellationKdTree::new(bounds);
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
            tess.add_wall(Wall::new_trefoil(cx, cy, cz, scale, tube_radius, 100, -10));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("trefoil/moctree", size), &size, |b, &s| {
            let mut tess = TessellationMoctree::new(bounds, 8);
            tess.add_wall(Wall::new_trefoil(cx, cy, cz, scale, tube_radius, 100, -10));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });

        group.bench_with_input(BenchmarkId::new("trefoil/kdtree", size), &size, |b, &s| {
            let mut tess = TessellationKdTree::new(bounds);
            tess.add_wall(Wall::new_trefoil(cx, cy, cz, scale, tube_radius, 100, -10));
            tess.random_generators(s);
            b.iter(|| {
                tess.calculate();
            })
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_distributions);
criterion_main!(benches);