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
    tests::{BenchStream, F64, F128},
    ProductSumcheck, Sumcheck,
};

pub mod validation;
use validation::{BenchArgs, AlgorithmLabel, FieldLabel, validate_and_format_command_line_args};

fn run_on_field<F: Field>(bench_args: BenchArgs) {
    let mut rng = ark_std::test_rng();
    let stream: BenchStream<F> =
        BenchStream::<F>::new(bench_args.num_variables);

    // switch on algorithm_label
    match bench_args.algorithm_label {
        AlgorithmLabel::Blendy => {
            let config: BlendyProverConfig<F, BenchStream<F>> =
                BlendyProverConfig::<F, BenchStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream,
                );
            Sumcheck::<F>::prove::<
                BenchStream<F>,
                BlendyProver<F, BenchStream<F>>,
            >(
                &mut BlendyProver::<F, BenchStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::VSBW => {
            let config: TimeProverConfig<F, BenchStream<F>> =
                TimeProverConfig::<F, BenchStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream,
                );
            Sumcheck::<F>::prove::<BenchStream<F>, TimeProver<F, BenchStream<F>>>(
                &mut TimeProver::<F, BenchStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::CTY => {
            let config: SpaceProverConfig<F, BenchStream<F>> =
                SpaceProverConfig::<F, BenchStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream,
                );
            Sumcheck::<F>::prove::<
                BenchStream<F>,
                SpaceProver<F, BenchStream<F>>,
            >(
                &mut SpaceProver::<F, BenchStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::ProductVSBW => {
            let config: TimeProductProverConfig<F, BenchStream<F>> =
                TimeProductProverConfig::<F, BenchStream<F>>::default(
                    stream.claimed_sum,
                    bench_args.num_variables,
                    stream.clone(),
                    stream,
                );
            ProductSumcheck::<F>::prove::<
                BenchStream<F>,
                TimeProductProver<F, BenchStream<F>>,
            >(
                &mut TimeProductProver::<F, BenchStream<F>>::new(config),
                &mut rng,
            );
        }
        AlgorithmLabel::ProductBlendy => {
            let config: BlendyProductProverConfig<F, BenchStream<F>> =
                BlendyProductProverConfig::<F, BenchStream<F>> {
                    claim: stream.claimed_sum,
                    num_variables: bench_args.num_variables,
                    num_stages: bench_args.stage_size,
                    stream_p: stream.clone(),
                    stream_q: stream,
                };
            ProductSumcheck::<F>::prove::<
                BenchStream<F>,
                BlendyProductProver<F, BenchStream<F>>,
            >(
                &mut BlendyProductProver::<F, BenchStream<F>>::new(config),
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