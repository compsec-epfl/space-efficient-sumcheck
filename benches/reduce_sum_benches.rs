#![feature(portable_simd)]

use ark_std::{
    simd::{cmp::SimdPartialOrd, u32x4, Mask, Simd},
    test_rng,
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_efficient_sumcheck::fields::{reduce_sum_naive, VecOps, M31, M31_MODULUS, aarch64_neon::reduce_sum_32_bit_modulus_asm};

// TODO (z-tech): this is the benchmark we should hit with both Neon and AVX
const LANES: usize = 4;
pub fn reduce_sum_packed(values: &[u32]) -> u32 {
    let packed_modulus: Simd<u32, LANES> = u32x4::splat(M31_MODULUS);
    let mut packed_sums1: Simd<u32, LANES> = u32x4::splat(0);
    let mut packed_sums2: Simd<u32, LANES> = u32x4::splat(0);
    let mut packed_sums3: Simd<u32, LANES> = u32x4::splat(0);
    let mut packed_sums4: Simd<u32, LANES> = u32x4::splat(0);
    for i in (0..values.len()).step_by(16) {
        let tmp_packed_sums_1: Simd<u32, LANES> =
            packed_sums1 + u32x4::from_slice(&values[i..i + 4]);
        let tmp_packed_sums_2: Simd<u32, LANES> =
            packed_sums2 + u32x4::from_slice(&values[i + 4..i + 8]);
        let tmp_packed_sums_3: Simd<u32, LANES> =
            packed_sums3 + u32x4::from_slice(&values[i + 8..i + 12]);
        let tmp_packed_sums_4: Simd<u32, LANES> =
            packed_sums4 + u32x4::from_slice(&values[i + 12..i + 16]);
        let is_mod_needed_1: Mask<i32, LANES> = tmp_packed_sums_1.simd_ge(packed_modulus);
        let is_mod_needed_2: Mask<i32, LANES> = tmp_packed_sums_2.simd_ge(packed_modulus);
        let is_mod_needed_3: Mask<i32, LANES> = tmp_packed_sums_3.simd_ge(packed_modulus);
        let is_mod_needed_4: Mask<i32, LANES> = tmp_packed_sums_4.simd_ge(packed_modulus);
        packed_sums1 =
            is_mod_needed_1.select(tmp_packed_sums_1 - packed_modulus, tmp_packed_sums_1);
        packed_sums2 =
            is_mod_needed_2.select(tmp_packed_sums_2 - packed_modulus, tmp_packed_sums_2);
        packed_sums3 =
            is_mod_needed_3.select(tmp_packed_sums_3 - packed_modulus, tmp_packed_sums_3);
        packed_sums4 =
            is_mod_needed_4.select(tmp_packed_sums_4 - packed_modulus, tmp_packed_sums_4);
    }
    reduce_sum_naive(&packed_sums1.to_array())
        + reduce_sum_naive(&packed_sums2.to_array())
        + reduce_sum_naive(&packed_sums3.to_array())
        + reduce_sum_naive(&packed_sums4.to_array())
}

fn reduce_sum_naive_bench(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u32())
        .collect();

    c.bench_function("reduce_sum_naive", |b| {
        b.iter(|| black_box(reduce_sum_naive(&random_values)))
    });
}

fn reduce_sum_simd_lib(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u32())
        .collect();

    c.bench_function("reduce_sum_simd_lib", |b| {
        b.iter(|| black_box(reduce_sum_packed(&random_values)))
    });
}

fn reduce_sum_neon_intrinsics(c: &mut Criterion) {
    let random_values: Vec<M31> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()))
        .collect();

    c.bench_function("reduce_sum_neon_intrinsics", |b| {
        b.iter(|| black_box(M31::reduce_sum(&random_values)))
    });
}

fn reduce_sum_neon_asm(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u32())
        .collect();

    c.bench_function("reduce_sum_neon_asm", |b| {
        b.iter(|| black_box(reduce_sum_32_bit_modulus_asm(&random_values, M31_MODULUS)))
    });
}

criterion_group!(
    benches,
    reduce_sum_naive_bench,
    reduce_sum_simd_lib,
    reduce_sum_neon_intrinsics,
    reduce_sum_neon_asm,
);
criterion_main!(benches);
