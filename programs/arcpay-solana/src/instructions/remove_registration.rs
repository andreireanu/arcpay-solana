use anchor_lang::prelude::*;
use crate::state::{Config, UserProfile};
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct RemoveRegistration<'info> {
    pub admin: Signer<'info>,

    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ ArcPayError::Unauthorized,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: wallet whose registration is being removed
    #[account(mut)]
    pub wallet: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"user", wallet.key().as_ref()],
        bump = user_profile.bump,
        close = wallet,
    )]
    pub user_profile: Account<'info, UserProfile>,
}

pub fn handler(_ctx: Context<RemoveRegistration>) -> Result<()> {
    Ok(())
}
