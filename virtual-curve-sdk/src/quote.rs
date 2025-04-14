use anyhow::{ensure, Context, Result};
use solana_sdk::pubkey::Pubkey;
use virtual_curve::{
    activation_handler::ActivationType,
    params::swap::TradeDirection,
    state::{fee::FeeMode, PoolConfig, PoolFeesConfig, VirtualPool, SwapResult},
};

pub fn quote_exact_in(
    virtual_pool: &VirtualPool,
    config: &PoolConfig,
    swap_base_for_quote: bool,
    current_timestamp: u64,
    current_slot: u64,
    transfer_fee_excluded_amount_in: u64, // must be calculated from outside
    has_referral: bool,
) -> Result<SwapResult> {
    let mut virtual_pool = *virtual_pool;

    ensure!(
        !virtual_pool.is_curve_complete(config.migration_quote_threshold),
        "virtual pool is completed"
    );

    ensure!(transfer_fee_excluded_amount_in > 0, "amount is zero");

    virtual_pool.update_pre_swap(config, current_timestamp)?;
    let activation_type =
        ActivationType::try_from(config.activation_type).context("invalid activation type")?;
    let current_point = match activation_type {
        ActivationType::Slot => current_slot,
        ActivationType::Timestamp => current_timestamp,
    };

    let trade_direction = if swap_base_for_quote {
        TradeDirection::BaseToQuote
    } else {
        TradeDirection::QuoteToBase
    };
    let fee_mode = &FeeMode::get_fee_mode(config.collect_fee_mode, trade_direction, has_referral)?;
    let swap_result = virtual_pool.get_swap_result(
        &config,
        transfer_fee_excluded_amount_in,
        fee_mode,
        trade_direction,
        current_point,
        virtual_pool.activation_point,
        &virtual_pool.volatility_tracker,
    )?;

    Ok(swap_result)
}

pub fn get_fee_mint(
    config: &PoolConfig,
    virtual_pool: &VirtualPool,
    swap_base_for_quote: bool,
    has_referral: bool,
) -> Result<Pubkey> {
    let trade_direction = if swap_base_for_quote {
        TradeDirection::BaseToQuote
    } else {
        TradeDirection::QuoteToBase
    };

    // Calculate the fee mode based on config, direction, and referral
    let fee_mode = FeeMode::get_fee_mode(config.collect_fee_mode, trade_direction, has_referral)
        .context("Failed to determine fee mode")?; // Use anyhow::Context for error handling

    // Determine the fee mint based on the fee mode
    let fee_mint = if fee_mode.fees_on_base_token {
        virtual_pool.base_mint
    } else {
        config.quote_mint // Quote mint is stored in the config
    };

    Ok(fee_mint)
}
