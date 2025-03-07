#include <metal_stdlib>
using namespace metal;

kernel void reduce_sum_mod(device const uint *input       [[ buffer(0) ]],
                           constant uint &modulus         [[ buffer(1) ]],
                           device uint *output            [[ buffer(2) ]],
                           constant uint &inputCount      [[ buffer(3) ]],
                           uint tid                       [[ thread_index_in_threadgroup ]],
                           uint gid                       [[ thread_position_in_grid ]],
                           uint numThreads                [[ threads_per_threadgroup ]])
{
    // Each thread loads one element if within bounds, otherwise 0.
    uint value = (gid < inputCount) ? input[gid] % modulus : 0;
    
    threadgroup uint localSums[256];
    localSums[tid] = value;
    threadgroup_barrier(mem_flags::mem_threadgroup);

    // Intra-group reduction with modulus at each step.
    for (uint stride = numThreads / 2; stride > 0; stride /= 2) {
        if (tid < stride) {
            localSums[tid] = (localSums[tid] + localSums[tid + stride]) % modulus;
        }
        threadgroup_barrier(mem_flags::mem_threadgroup);
    }

    // Write one partial result per threadgroup.
    if (tid == 0) {
        // Calculate the group index based on gid and threadgroup size.
        output[gid / numThreads] = localSums[0];
    }
}
