use ark_std;
use criterion::{criterion_group, criterion_main, Criterion};

use space_efficient_sumcheck::sumcheck::{SpaceProver, Sumcheck, SumcheckMultivariatePolynomial};

use ark_ff::{
    fields::Fp64,
    fields::{MontBackend, MontConfig},
};
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    DenseMVPolynomial,
};

#[derive(MontConfig)]
#[modulus = "19"]
#[generator = "2"]
struct FrConfig;

type TestField = Fp64<MontBackend<FrConfig, 1>>;
type TestPolynomial = multivariate::SparsePolynomial<TestField, SparseTerm>;

fn test_terms(num_terms: usize) -> Vec<(ark_ff::Fp<MontBackend<FrConfig, 1>, 1>, SparseTerm)> {
    let terms: Vec<(ark_ff::Fp<MontBackend<FrConfig, 1>, 1>, SparseTerm)> = vec![
        (
            TestField::from(4),
            multivariate::SparseTerm::new(vec![(0, 1)]),
        ),
        (
            TestField::from(7),
            multivariate::SparseTerm::new(vec![(1, 1), (2, 1)]),
        ),
        (
            TestField::from(2),
            multivariate::SparseTerm::new(vec![(0, 1)]),
        ),
        (
            TestField::from(13),
            multivariate::SparseTerm::new(vec![(1, 1)]),
        ),
        (
            TestField::from(27),
            multivariate::SparseTerm::new(vec![(3, 1)]),
        ),
        (
            TestField::from(29),
            multivariate::SparseTerm::new(vec![(1, 1), (4, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(1, 1), (4, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (5, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (6, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (7, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (8, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (9, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (10, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (11, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (12, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (13, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (14, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (15, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (16, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(2, 1), (17, 1)]),
        ),
        (
            TestField::from(18),
            multivariate::SparseTerm::new(vec![(3, 1), (18, 1)]),
        ),
    ];
    return terms[0..num_terms].to_vec();
}

fn test_polynomial(num_terms: usize) -> TestPolynomial {
    return TestPolynomial::from_coefficients_vec(num_terms, test_terms(num_terms));
}

fn sumcheck_benchmark(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();

    let polynomial = test_polynomial(14);
    let evaluations = polynomial.to_evaluations();
    c.bench_function("sumcheck_prove", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            let prover = SpaceProver::<TestField>::new(polynomial.num_vars, evaluations.clone());
            Sumcheck::prove(prover, &mut rng);
        });
    });
}

criterion_group!(benches, sumcheck_benchmark);
criterion_main!(benches);
