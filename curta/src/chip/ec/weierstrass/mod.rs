use num::{BigUint, Zero};
use serde::{Deserialize, Serialize};

use super::point::{AffinePoint, AffinePointRegister};
use super::{EllipticCurve, EllipticCurveParameters};
use crate::chip::builder::AirBuilder;
use crate::chip::field::instruction::FromFieldInstruction;
use crate::chip::field::parameters::{FieldParameters, MAX_NB_LIMBS};
use crate::chip::AirParameters;

pub mod biguint_operations;
pub mod bn254;
pub mod group;
pub mod slope;

/// Parameters that specify a short Weierstrass curve : y^2 = x^3 + ax + b.
pub trait WeierstrassParameters: EllipticCurveParameters {
    const A: [u16; MAX_NB_LIMBS];
    const B: [u16; MAX_NB_LIMBS];

    fn generator() -> AffinePoint<Self>;

    fn prime_group_order() -> BigUint;

    fn a_int() -> BigUint {
        let mut modulus = BigUint::zero();
        for (i, limb) in Self::A.iter().enumerate() {
            modulus += BigUint::from(*limb) << (16 * i);
        }
        modulus
    }

    fn b_int() -> BigUint {
        let mut modulus = BigUint::zero();
        for (i, limb) in Self::B.iter().enumerate() {
            modulus += BigUint::from(*limb) << (16 * i);
        }
        modulus
    }

    fn nb_scalar_bits() -> usize {
        Self::BaseField::NB_LIMBS * 16
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SWCurve<E>(pub E);

impl<E: WeierstrassParameters> EllipticCurveParameters for SWCurve<E> {
    type BaseField = E::BaseField;
}

impl<E: WeierstrassParameters> SWCurve<E> {
    pub fn generator() -> AffinePoint<SWCurve<E>> {
        let point = E::generator();

        AffinePoint::new(point.x, point.y)
    }

    pub fn a_int() -> BigUint {
        E::a_int()
    }

    pub fn b_int() -> BigUint {
        E::b_int()
    }
}

impl<L: AirParameters, E: WeierstrassParameters> EllipticCurve<L> for SWCurve<E>
where
    L::Instruction: FromFieldInstruction<E::BaseField>,
{
    fn ec_add(
        builder: &mut AirBuilder<L>,
        p: &AffinePointRegister<Self>,
        q: &AffinePointRegister<Self>,
    ) -> AffinePointRegister<Self> {
        builder.sw_add::<E>(p, q)
    }

    fn ec_double(
        builder: &mut AirBuilder<L>,
        p: &AffinePointRegister<Self>,
    ) -> AffinePointRegister<Self> {
        // TODO: might be expensive for no reason if doing more than one add in a row.
        // otherwise, there is no extra cost.
        let a = builder.fp_constant(&E::a_int());
        let three = builder.fp_constant(&BigUint::from(3u32));

        builder.sw_double::<E>(p, &a, &three)
    }

    fn ec_generator(builder: &mut AirBuilder<L>) -> AffinePointRegister<Self> {
        let generator = E::generator();

        let x = builder.fp_constant(&generator.x);
        let y = builder.fp_constant(&generator.y);

        AffinePointRegister::new(x, y)
    }
}
