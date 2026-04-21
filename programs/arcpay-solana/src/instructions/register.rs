use anchor_lang::prelude::*;
use crate::state::{UserProfile, WalletRegistered};

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

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Register>) -> Result<()> {
    let profile = &mut ctx.accounts.user_profile;
    profile.wallet = ctx.accounts.user.key();
    profile.bump = ctx.bumps.user_profile;

    emit!(WalletRegistered {
        user_profile: ctx.accounts.user_profile.key(),
        wallet: ctx.accounts.user.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
