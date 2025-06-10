use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};

use crate::{
    const_pda,
    state::{PoolConfig, VirtualPool},
    token::transfer_from_pool,
    EvtClaimTradingFee,
};

/// Accounts for partner to claim fees
#[event_cpi]
#[derive(Accounts)]
pub struct ClaimTradingFeesCtx<'info> {
    /// CHECK: pool authority
    #[account(
        address = const_pda::pool_authority::ID
    )]
    pub pool_authority: UncheckedAccount<'info>,

    #[account(has_one=quote_mint, has_one=fee_claimer)]
    pub config: AccountLoader<'info, PoolConfig>,

    #[account(
        mut,
        has_one = base_vault,
        has_one = quote_vault,
        has_one = base_mint,
        has_one = config,
    )]
    pub pool: AccountLoader<'info, VirtualPool>,

    /// The treasury token a account
    #[account(mut)]
    pub token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The treasury token b account
    #[account(mut)]
    pub token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for input token
    #[account(mut, token::token_program = token_base_program, token::mint = base_mint)]
    pub base_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The vault token account for output token
    #[account(mut, token::token_program = token_quote_program, token::mint = quote_mint)]
    pub quote_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token a
    pub base_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token b
    pub quote_mint: Box<InterfaceAccount<'info, Mint>>,

    pub fee_claimer: Signer<'info>,

    /// Token a program
    pub token_base_program: Interface<'info, TokenInterface>,

    /// Token b program
    pub token_quote_program: Interface<'info, TokenInterface>,
}

/// Partner claim fees.
pub fn handle_claim_trading_fee(
    ctx: Context<ClaimTradingFeesCtx>,
    max_base_amount: u64,
    max_quote_amount: u64,
) -> Result<()> {
    let mut pool = ctx.accounts.pool.load_mut()?;
    let (token_base_amount, token_quote_amount) =
        pool.claim_partner_trading_fee(max_base_amount, max_quote_amount)?;

    transfer_from_pool(
        ctx.accounts.pool_authority.to_account_info(),
        &ctx.accounts.base_mint,
        &ctx.accounts.base_vault,
        &ctx.accounts.token_a_account,
        &ctx.accounts.token_base_program,
        token_base_amount,
        const_pda::pool_authority::BUMP,
    )?;

    transfer_from_pool(
        ctx.accounts.pool_authority.to_account_info(),
        &ctx.accounts.quote_mint,
        &ctx.accounts.quote_vault,
        &ctx.accounts.token_b_account,
        &ctx.accounts.token_quote_program,
        token_quote_amount,
        const_pda::pool_authority::BUMP,
    )?;

    emit_cpi!(EvtClaimTradingFee {
        pool: ctx.accounts.pool.key(),
        token_base_amount,
        token_quote_amount
    });
    Ok(())
}
