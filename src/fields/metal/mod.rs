use metal::*;
use std::mem;

const SHADER_SOURCE: &str = include_str!("./shaders/reduce_sum.metal");

pub fn reduce_sum_32_bit_modulus_metal(values: &[u32], modulus: u32) -> u32 {
    let device = Device::system_default().expect("No system default Metal device");
    let command_queue = device.new_command_queue();

    // Load and compile the Metal shader.
    let options = CompileOptions::new();
    let library = device
        .new_library_with_source(&SHADER_SOURCE, &options)
        .expect("Failed to compile Metal shader");
    let kernel = library
        .get_function("reduce_sum_mod", None)
        .expect("Failed to get kernel function");
    let pipeline_state = device
        .new_compute_pipeline_state_with_function(&kernel)
        .expect("Failed to create pipeline state");

    // Copy input values into a GPU buffer.
    let mut current_buffer = device.new_buffer_with_data(
        values.as_ptr() as *const _,
        (values.len() * mem::size_of::<u32>()) as u64,
        MTLResourceOptions::CPUCacheModeDefaultCache,
    );

    let modulus_buffer = device.new_buffer_with_data(
        &modulus as *const u32 as *const _,
        mem::size_of::<u32>() as u64,
        MTLResourceOptions::CPUCacheModeDefaultCache,
    );

    let threadgroup_size = 256;
    let mut current_count = values.len();

    // Multi-pass reduction loop.
    while current_count > 1 {
        let num_threadgroups = (current_count + threadgroup_size - 1) / threadgroup_size;
        let partial_buffer_size = (num_threadgroups * mem::size_of::<u32>()) as u64;
        let partial_buffer =
            device.new_buffer(partial_buffer_size, MTLResourceOptions::StorageModeShared);

        // Create a buffer for the current count.
        let current_count_buffer = device.new_buffer_with_data(
            &current_count as *const usize as *const _,
            mem::size_of::<usize>() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );

        let command_buffer = command_queue.new_command_buffer();
        let encoder = command_buffer.new_compute_command_encoder();

        encoder.set_compute_pipeline_state(&pipeline_state);
        encoder.set_buffer(0, Some(&current_buffer), 0);
        encoder.set_buffer(1, Some(&modulus_buffer), 0);
        encoder.set_buffer(2, Some(&partial_buffer), 0);
        // Pass the current count (as a u32, adjust types as needed).
        let count_as_u32 = current_count as u32;
        let count_buffer = device.new_buffer_with_data(
            &count_as_u32 as *const u32 as *const _,
            mem::size_of::<u32>() as u64,
            MTLResourceOptions::CPUCacheModeDefaultCache,
        );
        encoder.set_buffer(3, Some(&count_buffer), 0);

        let tg_size = MTLSize {
            width: threadgroup_size as u64,
            height: 1,
            depth: 1,
        };
        let grid_size = MTLSize {
            width: current_count as u64,
            height: 1,
            depth: 1,
        };
        let threadgroups = MTLSize {
            width: num_threadgroups as u64,
            height: 1,
            depth: 1,
        };

        encoder.dispatch_thread_groups(threadgroups, tg_size);
        encoder.end_encoding();
        command_buffer.commit();
        command_buffer.wait_until_completed();

        // Prepare for next pass: the partial_buffer now contains 'num_threadgroups' elements.
        current_count = num_threadgroups;
        current_buffer = partial_buffer;
    }

    // Final result is in the first element of current_buffer.
    unsafe {
        let result_ptr = current_buffer.contents() as *const u32;
        *result_ptr
    }
}

mod tests {
    use crate::fields::{
        m31::{M31, M31_MODULUS},
        metal::reduce_sum_32_bit_modulus_metal,
        vec_ops::VecOps,
    };
    use ark_ff::{Field, One, Zero};
    use ark_std::{rand::Rng, test_rng};

    #[test]
    fn inverse_correctness() {
        let a = M31::from(2);
        assert_eq!(M31::from(1073741824), a.inverse().unwrap());
    }

    #[test]
    fn reduce_sum_correctness() {
        fn reduce_sum_sanity(vec: &[M31]) -> M31 {
            M31::from(vec.iter().fold(M31::zero(), |acc, &x| (acc + x)))
        }

        let mut rng = test_rng();
        let random_field_values: Vec<M31> = (0..1 << 13).map(|_| M31::rand(&mut rng)).collect();
        let random_field_values_u32: Vec<u32> =
            random_field_values.iter().map(|x| x.to_u32()).collect();
        let exp = reduce_sum_sanity(&random_field_values).to_u32();
        assert_eq!(
            exp,
            reduce_sum_32_bit_modulus_metal(&random_field_values_u32, M31_MODULUS)
        );
    }
}
