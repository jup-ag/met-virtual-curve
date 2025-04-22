//! Fees module includes information about fee charges
use crate::constants::fee::{
    FEE_DENOMINATOR, HOST_FEE_PERCENT, MAX_BASIS_POINT, MAX_FEE_NUMERATOR, MIN_FEE_NUMERATOR,
    PROTOCOL_FEE_PERCENT,
};
use crate::constants::{BASIS_POINT_MAX, BIN_STEP_BPS_DEFAULT, BIN_STEP_BPS_U128_DEFAULT, U24_MAX};
use crate::error::PoolError;
use crate::safe_math::SafeMath;
use crate::state::{BaseFeeConfig, DynamicFeeConfig, PoolFeesConfig};
use anchor_lang::prelude::*;

/// Information regarding fee charges
#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct PoolFeeParameters {
    /// Base fee
    pub base_fee: BaseFeeParameters,
    /// dynamic fee
    pub dynamic_fee: Option<DynamicFeeParameters>,
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct BaseFeeParameters {
    pub cliff_fee_numerator: u64,
    pub number_of_period: u16,
    pub period_frequency: u64,
    pub reduction_factor: u64,
    pub fee_scheduler_mode: u8,
}

impl BaseFeeParameters {
    fn validate(&self) -> Result<()> {
        let base_fee_config = self.to_base_fee_config();

        let min_fee_numerator = base_fee_config.get_min_base_fee_numerator()?;
        let max_fee_numerator = base_fee_config.get_max_base_fee_numerator();
        validate_fee_fraction(min_fee_numerator, FEE_DENOMINATOR)?;
        validate_fee_fraction(max_fee_numerator, FEE_DENOMINATOR)?;
        require!(
            min_fee_numerator >= MIN_FEE_NUMERATOR && max_fee_numerator <= MAX_FEE_NUMERATOR,
            PoolError::ExceedMaxFeeBps
        );
        Ok(())
    }

    pub fn to_base_fee_config(&self) -> BaseFeeConfig {
        BaseFeeConfig {
            cliff_fee_numerator: self.cliff_fee_numerator,
            number_of_period: self.number_of_period,
            period_frequency: self.period_frequency,
            reduction_factor: self.reduction_factor,
            fee_scheduler_mode: self.fee_scheduler_mode,
            ..Default::default()
        }
    }
}

impl PoolFeeParameters {
    pub fn to_pool_fees_config(&self) -> PoolFeesConfig {
        let &PoolFeeParameters {
            base_fee,
            dynamic_fee,
        } = self;
        if let Some(dynamic_fee) = dynamic_fee {
            PoolFeesConfig {
                base_fee: base_fee.to_base_fee_config(),
                protocol_fee_percent: PROTOCOL_FEE_PERCENT,
                referral_fee_percent: HOST_FEE_PERCENT,
                dynamic_fee: dynamic_fee.to_dynamic_fee_config(),
                ..Default::default()
            }
        } else {
            PoolFeesConfig {
                base_fee: base_fee.to_base_fee_config(),
                protocol_fee_percent: PROTOCOL_FEE_PERCENT,
                referral_fee_percent: HOST_FEE_PERCENT,
                ..Default::default()
            }
        }
    }
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, InitSpace, Default)]
pub struct DynamicFeeParameters {
    pub bin_step: u16,
    pub bin_step_u128: u128,
    pub filter_period: u16,
    pub decay_period: u16,
    pub reduction_factor: u16,
    pub max_volatility_accumulator: u32,
    pub variable_fee_control: u32,
}

impl DynamicFeeParameters {
    fn to_dynamic_fee_config(&self) -> DynamicFeeConfig {
        DynamicFeeConfig {
            initialized: 1,
            bin_step: self.bin_step,
            filter_period: self.filter_period,
            decay_period: self.decay_period,
            reduction_factor: self.reduction_factor,
            bin_step_u128: self.bin_step_u128,
            max_volatility_accumulator: self.max_volatility_accumulator,
            variable_fee_control: self.variable_fee_control,
            ..Default::default()
        }
    }

    pub fn validate(&self) -> Result<()> {
        // force all bin_step as 1 bps for first version
        require!(
            self.bin_step == BIN_STEP_BPS_DEFAULT,
            PoolError::InvalidInput
        );
        require!(
            self.bin_step_u128 == BIN_STEP_BPS_U128_DEFAULT,
            PoolError::InvalidInput
        );

        // filter period < t < decay period
        require!(
            self.filter_period < self.decay_period,
            PoolError::InvalidInput
        );

        // reduction factor decide the decay rate of variable fee, max reduction_factor is BASIS_POINT_MAX = 100% reduction
        require!(
            self.reduction_factor <= BASIS_POINT_MAX as u16,
            PoolError::InvalidInput
        );

        // prevent program overflow
        require!(
            self.variable_fee_control <= U24_MAX,
            PoolError::InvalidInput
        );
        require!(
            self.max_volatility_accumulator <= U24_MAX,
            PoolError::InvalidInput
        );

        Ok(())
    }
}

/// Helper function for calculating swap fee
pub fn calculate_fee(
    token_amount: u128,
    fee_numerator: u128,
    fee_denominator: u128,
) -> Option<u128> {
    if fee_numerator == 0 || token_amount == 0 {
        Some(0)
    } else {
        let fee = token_amount
            .checked_mul(fee_numerator)?
            .checked_div(fee_denominator)?;
        if fee == 0 {
            Some(1) // minimum fee of one token
        } else {
            Some(fee)
        }
    }
}

pub fn validate_fee_fraction(numerator: u64, denominator: u64) -> Result<()> {
    if denominator == 0 || numerator >= denominator {
        Err(PoolError::InvalidFee.into())
    } else {
        Ok(())
    }
}

/// Convert fees numerator and denominator to BPS. Minimum 1 bps, Maximum 10_000 bps. 0.01% -> 100%
pub fn to_bps(numerator: u128, denominator: u128) -> Result<u64> {
    let bps = numerator
        .safe_mul(MAX_BASIS_POINT.into())?
        .safe_div(denominator)?;
    Ok(u64::try_from(bps).map_err(|_| PoolError::TypeCastFailed)?)
}

impl PoolFeeParameters {
    /// Validate that the fees are reasonable
    pub fn validate(&self) -> Result<()> {
        self.base_fee.validate()?;

        if let Some(dynamic_fee) = self.dynamic_fee {
            dynamic_fee.validate()?;
        }

        Ok(())
    }
}
