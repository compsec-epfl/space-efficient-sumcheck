use ark_ff::{
    fields::Fp64,
    fields::{MontBackend, MontConfig},
    Field,
};
use ark_poly::{
    multivariate::{self, SparseTerm, Term},
    DenseMVPolynomial,
};
use criterion::{criterion_group, criterion_main, Criterion};

use space_efficient_sumcheck::{
    multilinear_extensions::{lagrange_polynomial, vsbw_interpolation},
    sumcheck::SumcheckMultivariatePolynomial,
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

fn lagrange_polynomial_benchmark(c: &mut Criterion) {
    c.bench_function("lagrange_polynomial", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            let x: Vec<TestField> = vec![
                TestField::ZERO,
                TestField::ONE,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ONE,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
                TestField::ZERO,
            ];
            let w: Vec<TestField> = vec![
                TestField::from(3),
                TestField::from(2),
                TestField::from(1),
                TestField::from(4),
                TestField::from(2),
                TestField::from(1),
                TestField::from(3),
                TestField::from(4),
                TestField::from(2),
                TestField::from(1),
                TestField::from(4),
                TestField::from(1),
                TestField::ZERO,
                TestField::ZERO,
                TestField::from(3),
                TestField::from(3),
            ];
            lagrange_polynomial(&x, &w);
        });
    });
}

fn vsbw_interpolation_benchmark(c: &mut Criterion) {
    let polynomial = test_polynomial(15);
    let evals = polynomial.to_evaluations();
    let r: Vec<TestField> = vec![
        TestField::from(3),
        TestField::from(2),
        TestField::from(1),
        TestField::from(4),
        TestField::from(2),
        TestField::from(1),
        TestField::from(3),
        TestField::from(4),
        TestField::from(2),
        TestField::from(1),
        TestField::from(4),
        TestField::from(1),
        TestField::ZERO,
        TestField::ZERO,
        TestField::from(3),
    ];
    c.bench_function("vsbw_interpolation", |b: &mut criterion::Bencher<'_>| {
        b.iter(|| {
            vsbw_interpolation(&evals, &r);
        });
    });
}

criterion_group!(
    benches,
    lagrange_polynomial_benchmark,
    vsbw_interpolation_benchmark
);
criterion_main!(benches);
