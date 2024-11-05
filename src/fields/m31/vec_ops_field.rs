use crate::fields::{
    m31::{M31, M31_MODULUS},
    VecOpsField,
};

impl VecOpsField for M31 {
    fn reduce_sum(vec: &[M31]) -> Self {
        let reduced_sum: u32 = vec.iter().fold(0, |acc, &x| {
            let sum = acc + x.to_u32();
            if sum < M31_MODULUS {
                return sum;
            } else {
                return sum - M31_MODULUS;
            }
        });
        Self { value: reduced_sum }
    }

    fn scalar_mult(values: &mut [Self], scalar: M31) {
        for elem in values.iter_mut() {
            *elem = *elem * scalar;
        }
    }
}
