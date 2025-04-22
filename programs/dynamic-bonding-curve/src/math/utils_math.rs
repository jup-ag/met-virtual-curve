use anchor_lang::prelude::Result;
use num_traits::cast::FromPrimitive;
use ruint::aliases::U256;

use crate::{
    safe_math::SafeMath,
    u128x128_math::{mul_shr, shl_div, Rounding},
    PoolError,
};

/// safe_mul_shr_cast
#[inline]
pub fn safe_mul_shr_cast<T: FromPrimitive>(x: u128, y: u128, offset: u8) -> Result<T> {
    T::from_u128(mul_shr(x, y, offset).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}

#[inline]
pub fn safe_mul_div_cast_u64<T: FromPrimitive>(
    x: u64,
    y: u64,
    denominator: u64,
    rounding: Rounding,
) -> Result<T> {
    let prod = u128::from(x).safe_mul(y.into())?;
    let denominator: u128 = denominator.into();

    let result = match rounding {
        Rounding::Up => prod
            .safe_add(denominator)?
            .safe_sub(1)?
            .safe_div(denominator)?,
        Rounding::Down => prod.safe_div(denominator)?,
    };

    T::from_u128(result).ok_or_else(|| PoolError::TypeCastFailed.into())
}

#[inline]
pub fn safe_mul_div_cast_u128(x: u128, y: u128, denominator: u128) -> Result<u128> {
    let result = U256::from(x)
        .safe_mul(U256::from(y))?
        .safe_div(U256::from(denominator))?;
    Ok(result.try_into().map_err(|_| PoolError::TypeCastFailed)?)
}

#[inline]
pub fn safe_shl_div_cast<T: FromPrimitive>(
    x: u128,
    y: u128,
    offset: u8,
    rounding: Rounding,
) -> Result<T> {
    T::from_u128(shl_div(x, y, offset, rounding).ok_or_else(|| PoolError::MathOverflow)?)
        .ok_or_else(|| PoolError::TypeCastFailed.into())
}
