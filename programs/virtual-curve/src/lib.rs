#![allow(unexpected_cfgs)]

use anchor_lang::prelude::*;

#[macro_use]
pub mod macros;

pub mod instructions;
pub use instructions::*;
pub mod constants;
pub mod error;
pub mod state;
pub use error::*;
pub mod event;
pub use event::*;
pub mod utils;
pub use utils::*;
pub mod math;
pub use math::*;
pub mod curve;
pub mod tests;

pub mod params;

#[cfg(feature = "local")]
declare_id!("2grmPSxKzQBhRrTRjWazVtMkRGFbFaqLakxGEZfnXB5u");

#[cfg(not(feature = "local"))]
declare_id!("virEFLZsQm1iFAs8py1XnziJ67gTzW2bfCWhxNPfccD");

#[program]
pub mod virtual_curve {
    use super::*;

    /// ADMIN FUNCTIONS ////
    pub fn create_claim_fee_operator(ctx: Context<CreateClaimFeeOperatorCtx>) -> Result<()> {
        instructions::handle_create_claim_fee_operator(ctx)
    }

    pub fn close_claim_fee_operator(ctx: Context<CloseClaimFeeOperatorCtx>) -> Result<()> {
        instructions::handle_close_claim_fee_operator(ctx)
    }

    pub fn claim_protocol_fee(ctx: Context<ClaimProtocolFeesCtx>) -> Result<()> {
        instructions::handle_claim_protocol_fee(ctx)
    }

    pub fn protocol_withdraw_surplus(ctx: Context<ProtocolWithdrawSurplusCtx>) -> Result<()> {
        instructions::handle_protocol_withdraw_surplus(ctx)
    }

    /// PARTNER FUNCTIONS ////
    pub fn create_partner_metadata(
        ctx: Context<CreatePartnerMetadataCtx>,
        metadata: CreatePartnerMetadataParameters,
    ) -> Result<()> {
        instructions::handle_create_partner_metadata(ctx, metadata)
    }
    pub fn create_config(
        ctx: Context<CreateConfigCtx>,
        config_parameters: ConfigParameters,
    ) -> Result<()> {
        instructions::handle_create_config(ctx, config_parameters)
    }
    pub fn claim_trading_fee(
        ctx: Context<ClaimTradingFeesCtx>,
        max_amount_a: u64,
        max_amount_b: u64,
    ) -> Result<()> {
        instructions::handle_claim_trading_fee(ctx, max_amount_a, max_amount_b)
    }

    pub fn partner_withdraw_surplus(ctx: Context<PartnerWithdrawSurplusCtx>) -> Result<()> {
        instructions::handle_partner_withdraw_surplus(ctx)
    }

    pub fn partner_claim_lp_from_meteora_dynamic_amm<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammClaimLpTokenCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm_partner_claim_lp_token(ctx)
    }

    /// POOL CREATOR FUNCTIONS ////
    pub fn initialize_virtual_pool_with_spl_token<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializeVirtualPoolWithSplTokenCtx<'info>>,
        params: InitializePoolParameters,
    ) -> Result<()> {
        instructions::handle_initialize_virtual_pool_with_spl_token(ctx, params)
    }

    pub fn initialize_virtual_pool_with_token2022<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, InitializeVirtualPoolWithToken2022Ctx<'info>>,
        params: InitializePoolParameters,
    ) -> Result<()> {
        instructions::handle_initialize_virtual_pool_with_token2022(ctx, params)
    }

    pub fn creator_claim_lp_from_meteora_dynamic_amm<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammClaimLpTokenCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm_creator_claim_lp_token(ctx)
    }

    /// TRADING BOTS FUNCTIONS ////
    pub fn swap(ctx: Context<SwapCtx>, params: SwapParameters) -> Result<()> {
        instructions::handle_swap(ctx, params)
    }

    /// PERMISSIONLESS FUNCTIONS ///
    pub fn migration_meteora_damm_create_metadata<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrationMeteoraDammCreateMetadataCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migration_meteora_damm_create_metadata(ctx)
    }

    pub fn migrate_meteora_damm<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm(ctx)
    }

    pub fn migrate_meteora_damm_lock_lp_token_for_creator<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammLockLpTokenCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm_lock_lp_token_for_creator(ctx)
    }

    pub fn migrate_meteora_damm_lock_lp_token_for_partner<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammLockLpTokenCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm_lock_lp_token_for_partner(ctx)
    }

    pub fn migrate_meteora_damm_partner_claim_lp_token<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammClaimLpTokenCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm_partner_claim_lp_token(ctx)
    }

    pub fn migrate_meteora_damm_creator_claim_lp_token<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateMeteoraDammClaimLpTokenCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_meteora_damm_creator_claim_lp_token(ctx)
    }

    // damm v2
    pub fn migration_damm_v2_create_metadata<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrationDammV2CreateMetadataCtx<'info>>,
    ) -> Result<()> {
        instructions::handle_migration_damm_v2_create_metadata(ctx)
    }

    pub fn migration_damm_v2<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, MigrateDammV2Ctx<'info>>,
    ) -> Result<()> {
        instructions::handle_migrate_damm_v2(ctx)
    }
}
