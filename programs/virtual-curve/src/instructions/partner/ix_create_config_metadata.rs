use anchor_lang::prelude::*;

use crate::{
    constants::seeds::CONFIG_METADATA_PREFIX, state::PoolConfigMetadata, EvtCreateConfigMetadata,
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateConfigMetadataParameters {
    // padding for future use
    pub padding: [u8; 48],
    pub name: String,
    pub website: String,
    pub logo: String,
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(metadata: CreateConfigMetadataParameters)]
pub struct CreateConfigMetadataCtx<'info> {
    /// The config metadata
    #[account(
        init,
        seeds = [
            CONFIG_METADATA_PREFIX.as_ref(),
            fee_claimer.key().as_ref()
        ],
        bump,
        payer = payer,
        space = 8 + PoolConfigMetadata::space(&metadata)
    )]
    pub config_metadata: Box<Account<'info, PoolConfigMetadata>>,
    /// Payer of the config metadata.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Fee claimer for partner
    pub fee_claimer: Signer<'info>,
    /// system program.
    pub system_program: Program<'info, System>,
}

pub fn handle_create_config_metadata(
    ctx: Context<CreateConfigMetadataCtx>,
    metadata: CreateConfigMetadataParameters,
) -> Result<()> {
    let config_metadata = &mut ctx.accounts.config_metadata;
    config_metadata.fee_claimer = ctx.accounts.fee_claimer.key();
    config_metadata.name = metadata.name.clone();
    config_metadata.website = metadata.website.clone();
    config_metadata.logo = metadata.logo.clone();
    emit_cpi!(EvtCreateConfigMetadata {
        config_metadata: ctx.accounts.config_metadata.key(),
        fee_claimer: ctx.accounts.fee_claimer.key(),
    });
    Ok(())
}
