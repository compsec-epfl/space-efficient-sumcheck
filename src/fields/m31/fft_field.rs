use ark_ff::FftField;

use crate::fields::m31::M31;

impl FftField for M31 {
    const GENERATOR: Self = M31 { value: 31 };

    const TWO_ADICITY: u32 = 27;

    const TWO_ADIC_ROOT_OF_UNITY: Self = M31 {
        value: 440564289,
    };

    const SMALL_SUBGROUP_BASE: Option<u32> = Some(3);

    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = Some(1);

    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = Some(M31 { value: 28629151 });

    fn get_root_of_unity(n: u64) -> Option<Self> {
        let m = match n {
            2 => 2013265920, // 1006632960
            3 => 1314723123, // 671088640
            5 => 645581151, // 402653184
            _ => 0,
        };
        if m == 0 {
            // root of unity of order n does not exist
            return None;
        }
        Some(M31 { value: m })
    }
}