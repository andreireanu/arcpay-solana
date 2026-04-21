use anchor_lang::prelude::*;
use crate::state::Config;
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ ArcPayError::Unauthorized,
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(ctx: Context<UpdateConfig>, commission_bps: u16) -> Result<()> {
    require!(commission_bps <= 10_000, ArcPayError::InvalidCommission);
    ctx.accounts.config.commission_bps = commission_bps;
    Ok(())
}
