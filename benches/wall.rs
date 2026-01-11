use criterion::{criterion_group, criterion_main, Criterion};
use vorothree::{BoundingBox, Tessellation, Wall};

const NUM_POINTS: usize = 1000;

fn benchmark_wall_plane(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut generators = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }

    let mut tess = Tessellation::new(bounds, 10, 10, 10);
    tess.set_generators(&generators);
    tess.add_wall(Wall::new_plane(50.0, 50.0, 50.0, 1.0, 1.0, 1.0, -11));

    c.bench_function(&format!("calculate_wall_plane_{}_points", NUM_POINTS), |b| {
        b.iter(|| {
            tess.calculate();
        })
    });
}

fn benchmark_wall_sphere(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut generators = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }

    let mut tess = Tessellation::new(bounds, 10, 10, 10);
    tess.set_generators(&generators);
    tess.add_wall(Wall::new_sphere(50.0, 50.0, 50.0, 40.0, -11));

    c.bench_function(&format!("calculate_wall_sphere_{}_points", NUM_POINTS), |b| {
        b.iter(|| {
            tess.calculate();
        })
    });
}

fn benchmark_wall_cylinder(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    
    let mut generators = Vec::with_capacity(NUM_POINTS * 3);
    for i in 0..NUM_POINTS {
        let v = (i as f64 / NUM_POINTS as f64) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }

    let mut tess = Tessellation::new(bounds, 10, 10, 10);
    tess.set_generators(&generators);
    tess.add_wall(Wall::new_cylinder(50.0, 50.0, 50.0, 0.0, 0.0, 1.0, 40.0, -11));

    c.bench_function(&format!("calculate_wall_cylinder_{}_points", NUM_POINTS), |b| {
        b.iter(|| {
            tess.calculate();
        })
    });
}

criterion_group!(benches, benchmark_wall_plane, benchmark_wall_sphere, benchmark_wall_cylinder);
criterion_main!(benches);