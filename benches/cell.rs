use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vorothree::{BoundingBox, Tessellation, AlgorithmGrid, CellFaces};

fn benchmark_cell_volume(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    let mut tess = Tessellation::<CellFaces, _>::new(bounds, AlgorithmGrid::new(10, 10, 10, &bounds));

    let mut generators: Vec<f64> = Vec::with_capacity(3000);
    for i in 0..10000 {
        let v: f64 = (i as f64 / 1000.0) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }
    
    tess.set_generators(&generators);
    tess.calculate();

    c.bench_function("cell_volume_10000", |b| {
        b.iter(|| {
            let count = tess.count_cells();
            for i in 0..count {
                if let Some(cell) = tess.get_cell(i) {
                    black_box(cell.volume());
                }
            }
        })
    });
}

fn benchmark_cell_centroid(c: &mut Criterion) {
    let bounds = BoundingBox::new(0.0, 0.0, 0.0, 100.0, 100.0, 100.0);
    let mut tess = Tessellation::<CellFaces, _>::new(bounds, AlgorithmGrid::new(10, 10, 10, &bounds));

    let mut generators: Vec<f64> = Vec::with_capacity(3000);
    for i in 0..10000 {
        let v: f64 = (i as f64 / 1000.0) * 100.0;
        generators.push(v);
        generators.push(v);
        generators.push(v);
    }
    
    tess.set_generators(&generators);
    tess.calculate();

    c.bench_function("cell_centroid_10000", |b| {
        b.iter(|| {
            let count = tess.count_cells();
            for i in 0..count {
                if let Some(cell) = tess.get_cell(i) {
                    black_box(cell.centroid());
                }
            }
        })
    });
}

criterion_group!(benches, benchmark_cell_volume, benchmark_cell_centroid);
criterion_main!(benches);