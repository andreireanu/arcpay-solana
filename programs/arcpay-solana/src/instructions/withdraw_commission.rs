use anchor_lang::prelude::*;
use crate::state::Config;
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct WithdrawCommission<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ ArcPayError::Unauthorized,
    )]
    pub config: Account<'info, Config>,
}

pub fn handler(ctx: Context<WithdrawCommission>, amount: u64) -> Result<()> {
    let config_info = ctx.accounts.config.to_account_info();
    let rent_minimum = Rent::get()?.minimum_balance(config_info.data_len());
    let available = config_info
        .lamports()
        .checked_sub(rent_minimum)
        .unwrap_or(0);

    require!(amount <= available, ArcPayError::InsufficientCommissionBalance);

    **config_info.try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.admin.to_account_info().try_borrow_mut_lamports()? += amount;

    Ok(())
}
