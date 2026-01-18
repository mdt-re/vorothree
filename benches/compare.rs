use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vorothree::{BoundingBox, Tessellation, TessellationMoctree};

const NUM_POINTS: usize = 1000;

fn benchmark_compare_insert(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    // Generate points along a diagonal for testing
    let mut generators = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }

    let mut group = c.benchmark_group("insert");

    group.bench_function("grid", |b| {
        let mut tess = Tessellation::new(bounds, 10, 10, 10);
        b.iter(|| {
            tess.set_generators(black_box(&generators));
        })
    });

    group.bench_function("moctree", |b| {
        let mut tess = TessellationMoctree::new(bounds, 8);
        b.iter(|| {
            tess.set_generators(black_box(&generators));
        })
    });

    group.finish();
}

fn benchmark_compare_calculate(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut generators = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }

    let mut group = c.benchmark_group("calculate");

    {
        let mut tess = Tessellation::new(bounds, 10, 10, 10);
        tess.set_generators(&generators);
        group.bench_function("grid", |b| {
            b.iter(|| {
                tess.calculate();
            })
        });
    }

    {
        let mut tess = TessellationMoctree::new(bounds, 8);
        tess.set_generators(&generators);
        group.bench_function("moctree", |b| {
            b.iter(|| {
                tess.calculate();
            })
        });
    }

    group.finish();
}

criterion_group!(benches, benchmark_compare_insert, benchmark_compare_calculate);
criterion_main!(benches);