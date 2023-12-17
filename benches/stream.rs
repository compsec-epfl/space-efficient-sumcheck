use ark_ff::{Field, fields::{Fp64, Fp128, MontBackend, MontConfig}};
use ark_bn254::Fr as BN254Field;

use space_efficient_sumcheck::{
    provers::{
        evaluation_stream::EvaluationStream, Prover, SpaceProver, TimeProver, TradeoffProver,
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

// BenchEvaluationStream just returns the field value of the index and uses constant memory
pub struct BenchEvaluationStream<F: Field> {
    pub num_variables: usize,
    pub claimed_sum: F,
}
impl<F: Field> BenchEvaluationStream<F> {
    pub fn new(num_variables: usize) -> Self {
        let hypercube_len = 2usize.pow(num_variables.try_into().unwrap());
        let mut claimed_sum: F = F::ZERO;
        for i in 0..hypercube_len {
            claimed_sum += F::from(i as u64);
        }
        Self {
            num_variables,
            claimed_sum,
        }
    }
    pub fn vec_of_field_to_usize(vec: Vec<F>) -> usize {
        // Reverse the vector to start from the least significant bit
        let reversed_vec: Vec<F> = vec.into_iter().rev().collect();

        // Calculate the decimal value
        let decimal_value: usize = reversed_vec
            .iter()
            .enumerate()
            .filter(|(_, &bit)| bit == F::ONE)
            .map(|(i, _)| 2usize.pow(i as u32))
            .sum();

        decimal_value
    }
}
impl<F: Field> EvaluationStream<F> for BenchEvaluationStream<F> {
    fn get_claimed_sum(&self) -> F {
        self.claimed_sum
    }
    fn get_evaluation(&self, point: Vec<F>) -> F {
        let index = BenchEvaluationStream::vec_of_field_to_usize(point);
        F::from(index as u64)
    }
    fn get_evaluation_from_index(&self, point: usize) -> F {
        F::from(point as u64)
    }
    fn get_num_variables(&self) -> usize {
        self.num_variables
    }
}

fn run_bench<F: Field, P: Prover<F>>(_c: &mut Criterion, mut prover: P, label: String) {
    let mut rng = ark_std::test_rng();
    epoch::advance().unwrap();
    let start_time = Instant::now();
    Sumcheck::prove(&mut prover, &mut rng);
    // Record the end time
    let end_time = Instant::now();

    // Calculate the elapsed time
    let elapsed_time = end_time - start_time;

    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{}, {:?}, {}, {}", label, elapsed_time.as_secs_f64(), allocated, resident);
}

fn vsbw_benches(c: &mut Criterion) {
    let max = 30;
    // 64 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<Field64>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw-fp64") + &format!("-{}", num_variables),
        );
    }
    // 128 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<Field128>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw-fp128") + &format!("-{}", num_variables),
        );
    }
    // bn254
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<BN254Field> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<BN254Field>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw-bn254") + &format!("-{}", num_variables),
        );
    }
}

fn cty_benches(c: &mut Criterion) {
    let max = 22;
    // 64 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> =
            BenchEvaluationStream::new(num_variables);
        let prover = SpaceProver::<Field64>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("cty-fp64") + &format!("-{}", num_variables),
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
        );
    }
}

fn tradeoff_k2_benches(c: &mut Criterion) {
    let max = 22;
    // 64 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> =
            BenchEvaluationStream::new(num_variables);
        let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 2);
        run_bench(
            c,
            prover,
            String::from("tradeoffk2-fp64") + &format!("-{}", num_variables),
        );
    }
    // 128 bit field
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field128> = BenchEvaluationStream::new(num_variables);
        let prover = TradeoffProver::<Field128>::new(Box::new(&stream), 2);
        run_bench(
            c,
            prover,
            String::from("tradeoffk2-fp128") + &format!("-{}", num_variables),
        );
    }
    // bn254
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<Field64> = BenchEvaluationStream::new(num_variables);
        let prover = TradeoffProver::<Field64>::new(Box::new(&stream), 2);
        run_bench(
            c,
            prover,
            String::from("tradeoffk2-bn254") + &format!("-{}", num_variables),
        );
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = vsbw_benches, cty_benches
}
criterion_main!(benches);
