use ark_ff::Field;
use ark_test_curves::bls12_381::Fr as BenchField;

use space_efficient_sumcheck::{
    provers::{
        evaluation_stream::EvaluationStream, Prover, SpaceProver, TimeProver, TradeoffProver,
    },
    Sumcheck,
};

use criterion::{criterion_group, criterion_main, Criterion};
use jemalloc_ctl::{epoch, stats};
use std::{thread, time::Duration};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

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

fn run_bench<F: Field, P: Prover<F>>(c: &mut Criterion, mut prover: P, label: String) {
    let mut rng = ark_std::test_rng();
    epoch::advance().unwrap();
    c.bench_function(&label, |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            Sumcheck::prove(&mut prover, &mut rng);
        });
    });
    let allocated = stats::allocated::read().unwrap();
    let resident = stats::resident::read().unwrap();
    println!("{} bytes allocated/{} bytes resident", allocated, resident);
    thread::sleep(Duration::from_secs(10));
}

fn all_benches(c: &mut Criterion) {
    let max = 16;
    // vsbw
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<BenchField> = BenchEvaluationStream::new(num_variables);
        let prover = TimeProver::<BenchField>::new(Box::new(&stream));
        run_bench(
            c,
            prover,
            String::from("vsbw") + &format!("-{}", num_variables),
        );
    }
    // tradeoff k=2
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<BenchField> = BenchEvaluationStream::new(num_variables);
        let prover = TradeoffProver::<BenchField>::new(Box::new(&stream), 2);
        if num_variables % 2 == 0 {
            run_bench(
                c,
                prover,
                String::from("tradoff2") + &format!("-{}", num_variables),
            );
        }
    }
    // tradeoff k=3
    for num_variables in 15..=max {
        let stream: BenchEvaluationStream<BenchField> = BenchEvaluationStream::new(num_variables);
        let prover = TradeoffProver::<BenchField>::new(Box::new(&stream), 3);
        if num_variables % 3 == 0 {
            run_bench(
                c,
                prover,
                String::from("tradeoff3") + &format!("-{}", num_variables),
            );
        }
        // cty
        for num_variables in 15..=max {
            let stream: BenchEvaluationStream<BenchField> =
                BenchEvaluationStream::new(num_variables);
            let prover = SpaceProver::<BenchField>::new(Box::new(&stream));
            run_bench(
                c,
                prover,
                String::from("cty") + &format!("-{}", num_variables),
            );
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = all_benches
}
criterion_main!(benches);
