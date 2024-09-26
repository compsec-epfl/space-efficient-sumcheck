use ark_ff::UniformRand;
use ark_std::test_rng;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_efficient_sumcheck::fields::baby_bear::BabyBear;

fn benchmark_reduce_sum_naive(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| BabyBear::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("reduce_sum", |b| {
        b.iter(|| black_box(BabyBear::reduce_sum(&random_values)))
    });
}

fn benchmark_reduce_sum(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| BabyBear::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("reduce_sum_packed", |b| {
        b.iter(|| black_box(BabyBear::reduce_sum_packed(&random_values)))
    });
}

criterion_group!(benches, benchmark_reduce_sum_naive, benchmark_reduce_sum);
criterion_main!(benches);
