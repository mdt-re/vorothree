use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vorothree::{BoundingBox, Tessellation};

const NUM_POINTS: usize = 1000;

fn benchmark_tessellation_update(c: &mut Criterion) {
    let bounds: BoundingBox = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    // Generate points along a diagonal for testing
    let mut generators: Vec<f64> = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v: f64 = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v); // x
        generators.push(v); // y
        generators.push(v); // z
    }

    c.bench_function(&format!("set_generators_{}_points", NUM_POINTS), |b| {
        // We reuse the tessellation instance to measure the update cost
        let mut tess: Tessellation = Tessellation::new(bounds, 10, 10, 10);
        
        b.iter(|| {
            // black_box prevents the compiler from optimizing away the arguments
            tess.set_generators(black_box(&generators));
        })
    });
}

fn benchmark_tessellation_calculate(c: &mut Criterion) {
    let bounds: BoundingBox = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    // Generate points along a diagonal for testing
    let mut generators: Vec<f64> = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v: f64 = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v); // x
        generators.push(v); // y
        generators.push(v); // z
    }

    let mut tess: Tessellation = Tessellation::new(bounds, 10, 10, 10);
    tess.set_generators(&generators);

    c.bench_function(&format!("calculate_{}_points", NUM_POINTS), |b| {
        b.iter(|| {
            tess.calculate();
        })
    });
}

criterion_group!(benches, benchmark_tessellation_update, benchmark_tessellation_calculate);
criterion_main!(benches);