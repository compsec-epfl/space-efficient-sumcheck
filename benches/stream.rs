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

fn run_bench<F: Field, P: Prover<F>>(
    _c: &mut Criterion,
    mut prover: P,
    label: String,
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
        "{}, {}, {}, {}",
        label,
        num_variables,
        elapsed_time.as_millis(),
        bytes_allocated
    );
}

fn warm_up(c: &mut Criterion) {
    let max = 22;
    // 64 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
        let prover = SpaceProver::<Field64>::new(Box::new(&stream));
        run_bench(c, prover, "warm_up".to_owned(), num_variables);
    }
}

fn vsbw_benches(c: &mut Criterion) {
    let max = 30;
    // 64 bit field
    for num_variables in 30..=max {
        let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<Field64>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw-fp64") + &format!("-{}", num_variables),
            num_variables,
        );
    }
    // 128 bit field
    for num_variables in 30..=max {
        let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<Field128>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw-fp128") + &format!("-{}", num_variables),
            num_variables,
        );
    }
    // bn254
    for num_variables in 30..=max {
        let stream: BenchEvaluationStream<BN254Field> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<BN254Field>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw-bn254") + &format!("-{}", num_variables),
            num_variables,
        );
    }
}

fn cty_benches(c: &mut Criterion) {
    let max = 28;
    // 64 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
        let prover = SpaceProver::<Field64>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("cty-fp64") + &format!("-{}", num_variables),
            num_variables,
        );
    }
    // 128 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
        let prover = SpaceProver::<Field128>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("cty-fp128") + &format!("-{}", num_variables),
            num_variables,
        );
    }
    // bn254
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
        let prover = SpaceProver::<Field64>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("cty-bn254") + &format!("-{}", num_variables),
            num_variables,
        );
    }
}

fn tradeoff_k2_benches(c: &mut Criterion) {
    let max = 30;
    // 64 bit field
    for num_variables in 15..=max {
        if num_variables % 2 == 0 {
            let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 2);
            run_bench(
                c,
                prover,
                String::from("tradeoffk2-fp64") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
    // 128 bit field
    for num_variables in 15..=max {
        if num_variables % 2 == 0 {
            let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field128>::new(Box::new(&stream), 2);
            run_bench(
                c,
                prover,
                String::from("tradeoffk2-fp128") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
    // bn254
    for num_variables in 15..=max {
        if num_variables % 2 == 0 {
            let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 2);
            run_bench(
                c,
                prover,
                String::from("tradeoffk2-bn254") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
}

fn tradeoff_k3_benches(c: &mut Criterion) {
    let max = 30;
    // 64 bit field
    for num_variables in 15..=max {
        if num_variables % 3 == 0 {
            let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 3);
            run_bench(
                c,
                prover,
                String::from("tradeoffk3-fp64") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
    // 128 bit field
    for num_variables in 15..=max {
        if num_variables % 3 == 0 {
            let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field128>::new(Box::new(&stream), 3);
            run_bench(
                c,
                prover,
                String::from("tradeoffk3-fp128") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
    // bn254
    for num_variables in 15..=max {
        if num_variables % 3 == 0 {
            let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 3);
            run_bench(
                c,
                prover,
                String::from("tradeoffk3-bn254") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
}

fn tradeoff_k4_benches(c: &mut Criterion) {
    let max = 30;
    // 64 bit field
    for num_variables in 15..=max {
        if num_variables % 4 == 0 {
            let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 4);
            run_bench(
                c,
                prover,
                String::from("tradeoffk4-fp64") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
    // 128 bit field
    for num_variables in 15..=max {
        if num_variables % 4 == 0 {
            let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field128>::new(Box::new(&stream), 4);
            run_bench(
                c,
                prover,
                String::from("tradeoffk4-fp128") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
    // bn254
    for num_variables in 15..=max {
        if num_variables % 4 == 0 {
            let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
            let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 4);
            run_bench(
                c,
                prover,
                String::from("tradeoffk4-bn254") + &format!("-{}", num_variables),
                num_variables,
            );
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = warm_up, tradeoff_k2_benches, tradeoff_k3_benches, tradeoff_k4_benches, vsbw_benches, cty_benches
}
criterion_main!(benches);
