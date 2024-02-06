use ark_bn254::Fr as BN254Field;
use ark_ff::{
    fields::{Fp128, Fp64, MontBackend, MontConfig},
    Field,
};
use space_efficient_sumcheck::{
    provers::{test_helpers::BenchEvaluationStream, SpaceProver, TimeProver, TradeoffProver},
    Sumcheck,
};
use std::env;

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

#[derive(Debug)]
enum FieldLabel {
    Field64,
    Field128,
    FieldBn254,
}

#[derive(Debug)]
enum AlgorithmLabel {
    CTY,
    VSBW,
    Tradeoff,
}

struct BenchArgs {
    field_label: FieldLabel,
    algorithm_label: AlgorithmLabel,
    num_variables: usize,
    stage_size: usize,
}

fn check_if_number(input_string: String) -> bool {
    match input_string.parse::<usize>() {
        Ok(_) => true,
        Err(_) => false,
    }
}

fn validate_and_format_command_line_args(argsv: Vec<String>) -> BenchArgs {
    // Check if the correct number of command line arguments is provided
    if argsv.len() != 5 {
        eprintln!(
            "Usage: {} field_label algorithm_label num_variables stage_size",
            argsv[0]
        );
        std::process::exit(1);
    }
    // algorithm label
    if !(argsv[1] == "CTY" || argsv[1] == "VSBW" || argsv[1] == "Tradeoff") {
        eprintln!(
            "Usage: {} field_label algorithm_label num_variables stage_size",
            argsv[0]
        );
        eprintln!("Invalid input: algorithm_label must be one of (CTY, VSBW, Tradeoff)");
        std::process::exit(1);
    }
    let algorithm_label = match argsv[1].as_str() {
        "CTY" => AlgorithmLabel::CTY,
        "VSBW" => AlgorithmLabel::VSBW,
        _ => AlgorithmLabel::Tradeoff, // this is checked in previous line
    };
    // field_label
    if !(argsv[2] == "Field64" || argsv[2] == "Field128" || argsv[2] == "FieldBn254") {
        eprintln!(
            "Usage: {} field_label algorithm_label num_variables stage_size",
            argsv[0]
        );
        eprintln!("Invalid input: field_label must be one of (Field64, Field128, FieldBn254)");
        std::process::exit(1);
    }
    let field_label = match argsv[2].as_str() {
        "Field64" => FieldLabel::Field64,
        "Field128" => FieldLabel::Field128,
        _ => FieldLabel::FieldBn254, // this is checked in previous line
    };
    // num_variables
    if !check_if_number(argsv[3].clone()) {
        eprintln!(
            "Usage: {} field_label algorithm_label num_variables stage_size",
            argsv[0]
        );
        eprintln!("Invalid input: num_variables must be a number");
        std::process::exit(1);
    }
    let num_variables = argsv[3].clone().parse::<usize>().unwrap();
    // stage_size
    if !check_if_number(argsv[4].clone()) {
        eprintln!(
            "Usage: {} field_label algorithm_label num_variables stage_size",
            argsv[0]
        );
        eprintln!("Invalid input: stage_size must be a number");
        std::process::exit(1);
    }
    let stage_size = argsv[4].clone().parse::<usize>().unwrap();
    return BenchArgs {
        field_label,
        algorithm_label,
        num_variables,
        stage_size,
    };
}

fn run_bench_on_field<F: Field>(bench_args: BenchArgs) {
    // get some predetermined randomness
    let mut rng = ark_std::test_rng();
    // create the bench stream O(1) memory usage
    let stream: BenchEvaluationStream<F> =
        BenchEvaluationStream::<F>::new(bench_args.num_variables);
    // switch on algorithm_label
    match bench_args.algorithm_label {
        AlgorithmLabel::CTY => {
            Sumcheck::prove(&mut SpaceProver::<F>::new(Box::new(&stream)), &mut rng);
        }
        AlgorithmLabel::VSBW => {
            Sumcheck::prove(&mut TimeProver::<F>::new(Box::new(&stream)), &mut rng);
        }
        AlgorithmLabel::Tradeoff => {
            Sumcheck::prove(
                &mut TradeoffProver::<F>::new(Box::new(&stream), bench_args.stage_size),
                &mut rng,
            );
        }
    };
}

fn main() {
    // Collect command line arguments
    let bench_args: BenchArgs = validate_and_format_command_line_args(env::args().collect());
    // Run the requested bench
    match bench_args.field_label {
        FieldLabel::Field64 => {
            run_bench_on_field::<Field64>(bench_args);
        }
        FieldLabel::Field128 => {
            run_bench_on_field::<Field128>(bench_args);
        }
        FieldLabel::FieldBn254 => {
            run_bench_on_field::<BN254Field>(bench_args);
        }
    };
}
