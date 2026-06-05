use crate::auth::auth_buy::verify_buy_auth;
use crate::state::{BuyCompleted, Config};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;
use anchor_lang::system_program;

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: recipient validated via signed message — backend verified seller identity
    #[account(mut)]
    pub seller: SystemAccount<'info>,

    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    /// CHECK: Instructions sysvar, used to verify the prepended ed25519 instruction
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<Buy>,
    seller_amount: u64,
    fee_amount: u64,
    offer_id: [u8; 16],
    expiry: i64,
) -> Result<()> {
    verify_buy_auth(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.config.backend_pubkey,
        &ctx.accounts.buyer.key(),
        &ctx.accounts.seller.key(),
        &offer_id,
        seller_amount,
        fee_amount,
        expiry,
    )?;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        seller_amount,
    )?;

    if fee_amount > 0 {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.config.to_account_info(),
                },
            ),
            fee_amount,
        )?;
    }

    emit!(BuyCompleted {
        offer_id,
        buyer: ctx.accounts.buyer.key(),
        seller: ctx.accounts.seller.key(),
        seller_amount,
        fee_amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
