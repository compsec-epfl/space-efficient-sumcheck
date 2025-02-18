use super::M31;

use ark_ff::FftField;

// TODO (z-tech): These might be correct we must verify each one

impl FftField for M31 {
    const GENERATOR: Self = M31 { value: 7 };

    const TWO_ADICITY: u32 = 1;

    const TWO_ADIC_ROOT_OF_UNITY: Self = M31 { value: 2147483646 };

    const SMALL_SUBGROUP_BASE: Option<u32> = Some(3);

    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = Some(1);

    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = Some(M31 { value: 6 });
}
