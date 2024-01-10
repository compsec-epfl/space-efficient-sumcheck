use ark_bn254::Fr as BN254Field;
use ark_ff::{
    fields::{Fp128, Fp64, MontBackend, MontConfig},
    Field,
};

use space_efficient_sumcheck::{
    provers::{
        test_helpers::BenchEvaluationStream, Prover, SpaceProver, TimeProver, TradeoffProver,
    },
    Sumcheck,
};

use criterion::{criterion_group, criterion_main, Criterion};
use jemalloc_ctl::{epoch, stats};
use std::time::Instant;

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

#[derive(MontConfig)]
#[modulus = "18446744069414584321"] // q = 2^64 - 2^32 + 1
#[generator = "2"]
pub struct FieldConfig64;
pub type Field64 = Fp64<MontBackend<FieldConfig64, 1>>;

#[derive(MontConfig)]
#[modulus = "143244528689204659050391023439224324689"] // q = 143244528689204659050391023439224324689
#[generator = "2"]
pub struct FieldConfig128;
pub type Field128 = Fp128<MontBackend<FieldConfig128, 2>>;
enum Algorithm {
    CTY,
    VSBW,
    Tradeoff,
}

fn run_bench<F: Field, P: Prover<F>>(
    _c: &mut Criterion,
    mut prover: P,
    label: &String,
    num_variables: usize,
) {
    let mut rng = ark_std::test_rng();
    epoch::advance().unwrap();
    let start_time = Instant::now();
    Sumcheck::prove(&mut prover, &mut rng);
    let end_time = Instant::now();
    let elapsed_time = end_time - start_time;
    let bytes_allocated = stats::allocated::read().unwrap();
    println!(
        "{}, {}, {}, {}, {}",
        label,
        num_variables,
        elapsed_time.as_millis(),
        num_variables,
        bytes_allocated
    );
}

fn run_group<F: Field>(
    c: &mut Criterion,
    algorithm: Algorithm,
    max_num_variables: usize,
    stage_size: Option<usize>,
    label: String,
) {
    for num_variables in 15..=max_num_variables {
        if stage_size == None || num_variables % stage_size.unwrap() == 0 {
            let stream: BenchEvaluationStream<F> = BenchEvaluationStream::<F>::new(num_variables);
            match algorithm {
                Algorithm::CTY => run_bench(
                    c,
                    SpaceProver::<F>::new(Box::new(&stream)),
                    &label,
                    num_variables,
                ),
                Algorithm::VSBW => run_bench(
                    c,
                    TimeProver::<F>::new(Box::new(&stream)),
                    &label,
                    num_variables,
                ),
                Algorithm::Tradeoff => run_bench(
                    c,
                    TradeoffProver::<F>::new(Box::new(&stream), stage_size.unwrap()),
                    &label,
                    num_variables,
                ),
            };
        }
    }
}

fn warm_up(c: &mut Criterion) {
    run_group::<Field64>(c, Algorithm::CTY, 22, None, String::from("warm_up"));
}

fn vsbw_benches(c: &mut Criterion) {
    let max_num_variables = 30;
    // 64 bit field
    run_group::<Field64>(
        c,
        Algorithm::VSBW,
        max_num_variables,
        None,
        String::from("vsbw-fp64"),
    );
    // 128 bit field
    run_group::<Field128>(
        c,
        Algorithm::VSBW,
        max_num_variables,
        None,
        String::from("vsbw-fp128"),
    );
    // bn254
    run_group::<BN254Field>(
        c,
        Algorithm::VSBW,
        max_num_variables,
        None,
        String::from("vsbw-bn254"),
    );
}

fn cty_benches(c: &mut Criterion) {
    let max_num_variables = 28;
    // 64 bit field
    run_group::<Field64>(
        c,
        Algorithm::CTY,
        max_num_variables,
        None,
        String::from("cty-fp64"),
    );
    // 128 bit field
    run_group::<Field128>(
        c,
        Algorithm::CTY,
        max_num_variables,
        None,
        String::from("cty-fp128"),
    );
    // bn254
    run_group::<BN254Field>(
        c,
        Algorithm::CTY,
        max_num_variables,
        None,
        String::from("cty-bn254"),
    );
}

fn tradeoff_k2_benches(c: &mut Criterion) {
    let max_num_variables = 26;
    // 64 bit field
    run_group::<Field64>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(2),
        String::from("tradeoffk2-fp64"),
    );
    // 128 bit field
    run_group::<Field128>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(2),
        String::from("tradeoffk2-fp128"),
    );
    // bn254
    run_group::<BN254Field>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(2),
        String::from("tradeoffk2-bn254"),
    );
}

fn tradeoff_k3_benches(c: &mut Criterion) {
    let max_num_variables = 30;
    // 64 bit field
    run_group::<Field64>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(3),
        String::from("tradeoffk3-fp64"),
    );
    // 128 bit field
    run_group::<Field128>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(3),
        String::from("tradeoffk3-fp128"),
    );
    // bn254
    run_group::<BN254Field>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(3),
        String::from("tradeoffk3-bn254"),
    );
}

fn tradeoff_k4_benches(c: &mut Criterion) {
    let max_num_variables = 30;
    // 64 bit field
    run_group::<Field64>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(4),
        String::from("tradeoffk4-fp64"),
    );
    // 128 bit field
    run_group::<Field128>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(4),
        String::from("tradeoffk4-fp128"),
    );
    // bn254
    run_group::<BN254Field>(
        c,
        Algorithm::Tradeoff,
        max_num_variables,
        Some(4),
        String::from("tradeoffk4-bn254"),
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = warm_up, tradeoff_k2_benches, tradeoff_k3_benches, tradeoff_k4_benches, vsbw_benches, cty_benches
}
criterion_main!(benches);
