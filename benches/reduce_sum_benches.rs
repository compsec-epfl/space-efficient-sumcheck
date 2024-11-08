use ark_ff::UniformRand;
use ark_std::test_rng;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_efficient_sumcheck::fields::{m31::M31, VecOpsField};

fn reduce_sum(c: &mut Criterion) {
    let random_values: Vec<M31> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()))
        .collect();

    c.bench_function("reduce_sum", |b| {
        b.iter(|| black_box(M31::reduce_sum(&random_values)))
    });
}

fn scalar_mult(c: &mut Criterion) {
    let mut random_values: Vec<M31> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()))
        .collect();

    c.bench_function("scalar_mult", |b| {
        b.iter(|| black_box(M31::scalar_mult(&mut random_values, M31::from(99999))))
    });
}

criterion_group!(benches, reduce_sum);
criterion_main!(benches);
