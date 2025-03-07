#![feature(portable_simd)]

use ark_std::{
    simd::{cmp::SimdPartialOrd, u32x32, Mask, Simd},
    test_rng,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_efficient_sumcheck::fields::{
    aarch64_neon::reduce_sum_32_bit_modulus_asm, metal::reduce_sum_32_bit_modulus_metal,
    reduce_sum_naive, VecOps, M31, M31_MODULUS,
};

const ONE_MILLION: usize = 1_000_000; // ~2^20
const SIXTEEN_MILLION: usize = 16_000_000; // ~2^24
const NUM_SAMPLES: usize = SIXTEEN_MILLION;

fn random_values() -> Vec<u32> {
    (0..NUM_SAMPLES).map(|_| M31::rand(&mut test_rng()).to_u32()).collect()
}

// TODO (z-tech): this is the benchmark we should hit with both Neon and AVX
pub fn reduce_sum_packed(values: &[u32]) -> u32 {
    const LANES: usize = 32;
    let packed_modulus: Simd<u32, LANES> = u32x32::splat(M31_MODULUS);
    let mut packed_sums: Simd<u32, LANES> = u32x32::splat(0);
    for i in (0..values.len()).step_by(LANES) {
        let tmp: Simd<u32, LANES> = packed_sums + u32x32::from_slice(&values[i..i + LANES]);
        let is_mod_needed: Mask<i32, LANES> = tmp.simd_ge(packed_modulus);
        packed_sums = is_mod_needed.select(tmp - packed_modulus, tmp);
    }
    reduce_sum_naive(&packed_sums.to_array())
}

fn reduce_sum_naive_bench(c: &mut Criterion) {
    c.bench_function("reduce_sum_naive", |b| {
        b.iter(|| black_box(reduce_sum_naive(&random_values())))
    });
}

fn reduce_sum_simd_lib(c: &mut Criterion) {
    c.bench_function("reduce_sum_simd_lib", |b| {
        b.iter(|| black_box(reduce_sum_packed(&random_values())))
    });
}

fn reduce_sum_neon_intrinsics(c: &mut Criterion) {
    let random_values: Vec<M31> = (0..NUM_SAMPLES)
        .map(|_| M31::rand(&mut test_rng()))
        .collect();

    c.bench_function("reduce_sum_neon_intrinsics", |b| {
        b.iter(|| black_box(M31::reduce_sum(&random_values)))
    });
}

fn reduce_sum_neon_asm(c: &mut Criterion) {
    c.bench_function("reduce_sum_neon_asm", |b| {
        b.iter(|| black_box(reduce_sum_32_bit_modulus_asm(&random_values(), M31_MODULUS)))
    });
}

fn reduce_sum_metal(c: &mut Criterion) {
    c.bench_function("reduce_sum_metal", |b| {
        b.iter(|| black_box(reduce_sum_32_bit_modulus_metal(&random_values(), M31_MODULUS)))
    });
}

criterion_group!(
    benches,
    reduce_sum_metal,
    reduce_sum_simd_lib,
    reduce_sum_naive_bench,
    reduce_sum_neon_intrinsics,
    reduce_sum_neon_asm,
);
criterion_main!(benches);
