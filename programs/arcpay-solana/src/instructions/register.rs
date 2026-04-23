use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use crate::auth::verify_backend_auth;
use crate::state::{Config, UserProfile, WalletRegistered};

#[derive(Accounts)]
pub struct Register<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + UserProfile::INIT_SPACE,
        seeds = [b"user", user.key().as_ref()],
        bump,
    )]
    pub user_profile: Account<'info, UserProfile>,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    /// CHECK: Instructions sysvar, used to verify the prepended ed25519 instruction
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Register>, expiry: i64, uuid: [u8; 16]) -> Result<()> {
    verify_backend_auth(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.config.backend_pubkey,
        &ctx.accounts.user.key(),
        &uuid,
        expiry,
    )?;

    let profile = &mut ctx.accounts.user_profile;
    profile.wallet = ctx.accounts.user.key();
    profile.bump = ctx.bumps.user_profile;

    emit!(WalletRegistered {
        user_profile: ctx.accounts.user_profile.key(),
        wallet: ctx.accounts.user.key(),
        uuid,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
