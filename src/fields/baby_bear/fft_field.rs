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

    fn get_root_of_unity(n: u64) -> Option<Self> {
        let m = match n {
            2 => 2147483646,
            3 => 1513477735,
            7 => 1205362885,
            11 => 1969212174,
            31 => 512,
            151 => 535044134,
            331 => 1761855083,
            _ => 0,
        };
        if m == 0 {
            // root of unity of order n does not exist
            return None;
        }
        Some(BabyBear { mod_value: m })
    }
}
