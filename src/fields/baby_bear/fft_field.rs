use super::BabyBear;
use ark_ff::FftField;

impl FftField for BabyBear {
    const GENERATOR: Self = BabyBear { mod_value: 7 };

    const TWO_ADICITY: u32 = 1;

    const TWO_ADIC_ROOT_OF_UNITY: Self = BabyBear {
        mod_value: 2147483646,
    };

    const SMALL_SUBGROUP_BASE: Option<u32> = Some(3);

    const SMALL_SUBGROUP_BASE_ADICITY: Option<u32> = Some(1);

    const LARGE_SUBGROUP_ROOT_OF_UNITY: Option<Self> = Some(BabyBear { mod_value: 6 });
}
