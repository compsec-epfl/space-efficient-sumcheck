use ark_bn254::Fr as BN254Field;
use ark_ff::Field;

use space_efficient_sumcheck::{
    multilinear::{
        BlendyProver, BlendyProverConfig, SpaceProver, SpaceProverConfig, TimeProver,
        TimeProverConfig,
    },
    multilinear_product::{
        BlendyProductProver, BlendyProductProverConfig, TimeProductProver, TimeProductProverConfig,
    },
    prover::{ProductProverConfig, Prover, ProverConfig},
    tests::{BenchEvaluationStream, F64, F128},
    ProductSumcheck, Sumcheck,
};

pub mod validation;
use validation::{BenchArgs, AlgorithmLabel, FieldLabel, validate_and_format_command_line_args};

fn run_on_field<F: Field>(bench_args: BenchArgs) {
    let mut rng = ark_std::test_rng();
    let stream: BenchEvaluationStream<F> =
        BenchEvaluationStream::<F>::new(bench_args.num_variables);

    // switch on algorithm_label
    match bench_args.algorithm_label {
        AlgorithmLabel::Blendy => {
            let config: BlendyProverConfig<F, BenchEvaluationStream<F>> =
                BlendyProverConfig::<F, BenchEvaluationStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream,
                );
            Sumcheck::<F>::prove::<
                BenchEvaluationStream<F>,
                BlendyProver<F, BenchEvaluationStream<F>>,
            >(
                &mut BlendyProver::<F, BenchEvaluationStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::VSBW => {
            let config: TimeProverConfig<F, BenchEvaluationStream<F>> =
                TimeProverConfig::<F, BenchEvaluationStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream,
                );
            Sumcheck::<F>::prove::<BenchEvaluationStream<F>, TimeProver<F, BenchEvaluationStream<F>>>(
                &mut TimeProver::<F, BenchEvaluationStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::CTY => {
            let config: SpaceProverConfig<F, BenchEvaluationStream<F>> =
                SpaceProverConfig::<F, BenchEvaluationStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream,
                );
            Sumcheck::<F>::prove::<
                BenchEvaluationStream<F>,
                SpaceProver<F, BenchEvaluationStream<F>>,
            >(
                &mut SpaceProver::<F, BenchEvaluationStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::ProductVSBW => {
            let config: TimeProductProverConfig<F, BenchEvaluationStream<F>> =
                TimeProductProverConfig::<F, BenchEvaluationStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream.clone(),
                    stream,
                );
            ProductSumcheck::<F>::prove::<
                BenchEvaluationStream<F>,
                TimeProductProver<F, BenchEvaluationStream<F>>,
            >(
                &mut TimeProductProver::<F, BenchEvaluationStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::ProductBlendy => {
            let config: BlendyProductProverConfig<F, BenchEvaluationStream<F>> =
                BlendyProductProverConfig::<F, BenchEvaluationStream<F>> {
                    claim: stream.claimed_sum,
                    num_variables: bench_args.num_variables,
                    num_stages: bench_args.stage_size,
                    stream_p: stream.clone(),
                    stream_q: stream,
                };
            ProductSumcheck::<F>::prove::<
                BenchEvaluationStream<F>,
                BlendyProductProver<F, BenchEvaluationStream<F>>,
            >(
                &mut BlendyProductProver::<F, BenchEvaluationStream<F>>::new(config),
                &mut rng,
            );
        }
    };
}

fn main() {
    // Collect command line arguments
    let bench_args: BenchArgs = validate_and_format_command_line_args(std::env::args().collect());
    // Run the requested bench
    match bench_args.field_label {
        FieldLabel::Field64 => {
            run_on_field::<F64>(bench_args);
        }
        FieldLabel::Field128 => {
            run_on_field::<F128>(bench_args);
        }
        FieldLabel::FieldBn254 => {
            run_on_field::<BN254Field>(bench_args);
        }
    };
}
