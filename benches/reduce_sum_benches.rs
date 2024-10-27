use ark_ff::UniformRand;
use ark_std::test_rng;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use space_efficient_sumcheck::fields::{baby_bear::BabyBear, m31::M31};

fn reduce_sum_naive(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("reduce_sum", |b| {
        b.iter(|| black_box(M31::reduce_sum(&random_values)))
    });
}

fn reduce_sum_packed(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("reduce_sum_packed", |b| {
        b.iter(|| black_box(M31::reduce_sum_packed(&random_values)))
    });
}

fn reduce_sum_packed_neon(c: &mut Criterion) {
    let random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("reduce_sum_packed", |b| {
        b.iter(|| black_box(M31::reduce_sum_packed_neon(&random_values)))
    });
}

fn batch_mult_normal(c: &mut Criterion) {
    let mut random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("batch_mult_normal", |b| {
        b.iter(|| black_box(M31::batch_mult_normal(&mut random_values, 99999)))
    });
}

fn batch_mult_trick(c: &mut Criterion) {
    let mut random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("batch_mult_trick", |b| {
        b.iter(|| black_box(M31::batch_mult_trick(&mut random_values, 99999)))
    });
}

fn batch_mult_trick_packed(c: &mut Criterion) {
    let mut random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("batch_mult_trick_packed", |b| {
        b.iter(|| black_box(M31::batch_mult_trick_packed(&mut random_values, 99999)))
    });
}

fn batch_mult_trick_parts(c: &mut Criterion) {
    let mut random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("batch_mult_trick_parts", |b| {
        b.iter(|| black_box(M31::batch_mult_parts(&mut random_values, 99999)))
    });
}

fn batch_sum_packed(c: &mut Criterion) {
    let mut random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("batch_sum_packed", |b| {
        b.iter(|| black_box(M31::batch_sum_packed(&mut random_values)))
    });
}

fn batch_mult_trick_parts_packed(c: &mut Criterion) {
    let mut random_values: Vec<u32> = (0..2_i32.pow(13))
        .map(|_| M31::rand(&mut test_rng()).to_u64() as u32)
        .collect();

    c.bench_function("batch_mult_trick_parts_packed", |b| {
        b.iter(|| {
            black_box(M31::batch_mult_trick_parts_packed(
                &mut random_values,
                99999,
            ))
        })
    });
}

// fn batch_mult_mont(c: &mut Criterion) {
//     let mut random_values: Vec<u32> = (0..2_i32.pow(13))
//         .map(|_| BabyBear::rand(&mut test_rng()).to_u64() as u32)
//         .collect();

//     c.bench_function("batch_mult_mont", |b| {
//         b.iter(|| black_box(BabyBear::batch_mult_mont(&mut random_values, 99999)))
//     });
// }

// fn batch_mult_parts(c: &mut Criterion) {
//     let mut random_values: Vec<u32> = (0..2_i32.pow(13))
//         .map(|_| BabyBear::rand(&mut test_rng()).to_u64() as u32)
//         .collect();

//     c.bench_function("batch_mult_parts", |b| {
//         b.iter(|| black_box(BabyBear::batch_mult_parts(&mut random_values, 99999)))
//     });
// }

// fn batch_mult_normal_packed(c: &mut Criterion) {
//     let mut random_values: Vec<u32> = (0..2_i32.pow(13))
//         .map(|_| BabyBear::rand(&mut test_rng()).to_u64() as u32)
//         .collect();

//     c.bench_function("batch_mult_normal_packed", |b| {
//         b.iter(|| {
//             black_box(BabyBear::batch_mult_normal_packed(
//                 &mut random_values,
//                 99999,
//             ))
//         })
//     });
// }

criterion_group!(
    benches,
    batch_mult_normal,
    batch_mult_trick,
    batch_mult_trick_parts,
    batch_mult_trick_packed,
    batch_mult_trick_parts_packed,
    // batch_mult_mont,
    // batch_mult_parts,
    // batch_mult_normal_packed,
    // reduce_sum_naive,
    // reduce_sum_packed,
    // reduce_sum_packed_neon,
);
criterion_main!(benches);
