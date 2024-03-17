use num::{BigUint, Num, Zero};
use serde::{Deserialize, Serialize};

use super::{SwCurve, WeierstrassParameters};
use crate::operations::field::params::{NB_BITS_PER_LIMB, NUM_LIMBS};
use crate::utils::ec::field::{FieldParameters, MAX_NB_LIMBS};
use crate::utils::ec::EllipticCurveParameters;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Bn254 curve parameter
pub struct Bn254Parameters;

pub type Bn254 = SwCurve<Bn254Parameters>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
/// Bn254 base field parameter
pub struct Bn254BaseField;

impl FieldParameters for Bn254BaseField {
    const NB_BITS_PER_LIMB: usize = NB_BITS_PER_LIMB;

    const NB_LIMBS: usize = NUM_LIMBS;

    const NB_WITNESS_LIMBS: usize = 2 * Self::NB_LIMBS - 2;

    // The modulus have been taken from starkyx. It's the little-endian `u8` representation of the
    // modulus.
    const MODULUS: [u8; MAX_NB_LIMBS] = [
        71, 253, 124, 216, 22, 140, 32, 60, 141, 202, 113, 104, 145, 106, 129, 151, 93, 88, 129,
        129, 182, 69, 80, 184, 41, 160, 49, 225, 114, 78, 100, 48,
    ];

    const WITNESS_OFFSET: usize = 1usize << 13;

    const NAME: &'static str = "Bn254BaseField";

    // The modulus have been taken from starkyx
    // https://github.com/succinctlabs/starkyx/blob/ad8eb4b/starkyx/src/chip/ec/weierstrass/bn254.rs#L25-L27
    fn modulus() -> BigUint {
        BigUint::from_str_radix(
            "21888242871839275222246405745257275088696311157297823662689037894645226208583",
            10,
        )
        .unwrap()
    }
}

impl EllipticCurveParameters for Bn254Parameters {
    type BaseField = Bn254BaseField;
}

impl WeierstrassParameters for Bn254Parameters {
    // All the constants have been taken from starkyx
    // https://github.com/succinctlabs/starkyx/blob/ad8eb4ba/starkyx/src/chip/ec/weierstrass/bn254.rs#L39-L70
    const A: [u16; MAX_NB_LIMBS] = [
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];

    const B: [u16; MAX_NB_LIMBS] = [
        3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ];
    fn generator() -> (BigUint, BigUint) {
        let x = BigUint::from(1u32);
        let y = BigUint::from(2u32);
        (x, y)
    }

    fn prime_group_order() -> num::BigUint {
        BigUint::from_str_radix(
            "21888242871839275222246405745257275088548364400416034343698204186575808495617",
            10,
        )
        .unwrap()
    }

    fn a_int() -> BigUint {
        BigUint::zero()
    }

    fn b_int() -> BigUint {
        BigUint::from(3u32)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::utils::ec::utils::biguint_from_limbs;

    #[test]
    fn test_weierstrass_biguint_scalar_mul() {
        assert_eq!(
            biguint_from_limbs(&Bn254BaseField::MODULUS),
            Bn254BaseField::modulus()
        );
    }
}
