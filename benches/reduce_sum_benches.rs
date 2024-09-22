#![feature(portable_simd)]

use std::simd::{u64x64, Simd};

use ark_ff::UniformRand;
use ark_std::test_rng;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_efficient_sumcheck::fields::m31::{M31, M31_MODULUS_U64}; // Import your type and the reduce_sum function

fn benchmark_reduce_sum_naive(c: &mut Criterion) {
    // Generate a random vector of u64 values to use in the benchmark
    // let mut rng = test_rng();
    // let vec: Vec<u64> = (0..10000).map(|_| M31::rand(&mut rng).to_u64()).collect();
    let values = [
        0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5,
        6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3,
        4, 5, 6, 7,
        0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5,
        6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3,
        4, 5, 6, 7,
    ];

    // Benchmark the reduce_sum function
    c.bench_function("reduce_sum_normal", |b| {
        b.iter(|| M31::reduce_sum_naive(black_box(&values)))
    });
}

fn benchmark_reduce_sum(c: &mut Criterion) {
    // Generate a random vector of u64 values to use in the benchmark
    // let mut rng = test_rng();
    // let vec: Vec<u64> = (0..10000).map(|_| M31::rand(&mut rng).to_u64()).collect();

    let mut sums: Simd<u64, 64> = u64x64::from_array([0; 64]);
    let modulus: Simd<u64, 64> = u64x64::from_array([M31_MODULUS_U64; 64]);
    let values = [
        0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5,
        6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3,
        4, 5, 6, 7,
        0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5,
        6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3,
        4, 5, 6, 7,
    ];

    // Benchmark the reduce_sum function
    c.bench_function("reduce_sum_simd", |b| {
        b.iter(|| {
            M31::reduce_sum(
                black_box(&mut sums),
                black_box(&modulus),
                black_box(&values),
            )
        })
    });
}

criterion_group!(benches, benchmark_reduce_sum, benchmark_reduce_sum_naive);
criterion_main!(benches);
