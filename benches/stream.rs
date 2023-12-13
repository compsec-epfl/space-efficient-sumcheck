use ark_ff::Field;
use ark_test_curves::bls12_381::Fr as BenchField;

use space_efficient_sumcheck::{
    provers::{evaluation_stream::EvaluationStream, SpaceProver, TimeProver, TradeoffProver},
    Sumcheck,
};

// bench specific stuff
use std::thread;
use std::time::Duration;
use criterion::{criterion_group, criterion_main, Criterion};
use jemalloc_ctl::{epoch, stats};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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

// fn run_bench<F: Field + std::convert::From<i32>, P: Prover<F>>(c: &mut Criterion) {
//     let mut rng = ark_std::test_rng();

//     let polynomial = test_polynomial(8);
//     let evaluations = polynomial.to_evaluations();
//     epoch::advance().unwrap();
//     c.bench_function("tradeoff_prover", |b: &mut criterion::Bencher<'_>| {
//         b.iter(|| {
//             let prover = P::new(evaluations.clone(), 4);
//             Sumcheck::prove(prover, &mut rng);
//         });
//     });
//     let allocated = stats::allocated::read().unwrap();
//     let resident = stats::resident::read().unwrap();
//     println!("{} bytes allocated/{} bytes resident", allocated, resident);
//     thread::sleep(Duration::from_secs(10));
// }

fn time_prover_benchmark(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();

    let stream: BenchEvaluationStream<BenchField> = BenchEvaluationStream::new(10);
    epoch::advance().unwrap();
    c.bench_function("time_prover", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            let prover = TimeProver::<BenchField>::new(Box::new(&stream));
            Sumcheck::prove(prover, &mut rng);
        });
    });
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident", allocated, resident);
    thread::sleep(Duration::from_secs(10));
}

fn space_prover_benchmark(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();

    let stream: BenchEvaluationStream<BenchField> = BenchEvaluationStream::new(10);
    epoch::advance().unwrap();
    c.bench_function("space_prover", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            let prover = SpaceProver::<BenchField>::new(Box::new(&stream));
            Sumcheck::prove(prover, &mut rng);
        });
    });
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident", allocated, resident);
    thread::sleep(Duration::from_secs(10));
}

fn tradeoff_prover_benchmark(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();

    let stream: BenchEvaluationStream<BenchField> = BenchEvaluationStream::new(10);
    epoch::advance().unwrap();
    c.bench_function("tradeoff_prover", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            let prover = TradeoffProver::<BenchField>::new(Box::new(&stream), 4);
            Sumcheck::prove(prover, &mut rng);
        });
    });
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident", allocated, resident);
    thread::sleep(Duration::from_secs(10));
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = tradeoff_prover_benchmark, space_prover_benchmark, time_prover_benchmark
}
criterion_main!(benches);
