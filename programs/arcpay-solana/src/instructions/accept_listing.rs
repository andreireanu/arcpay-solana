use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::{Config, Listing, ListingAccepted};
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct AcceptListing<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: receives listing price minus fee + rent, validated via has_one
    #[account(mut)]
    pub seller: SystemAccount<'info>,

    #[account(
        mut,
        has_one = seller @ ArcPayError::Unauthorized,
        constraint = listing.is_active @ ArcPayError::ListingNotActive,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,

    #[account(mut, seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptListing>) -> Result<()> {
    let price = ctx.accounts.listing.price_lamports;
    let commission_bps = ctx.accounts.config.commission_bps;
    let seller = ctx.accounts.listing.seller;
    let buyer = ctx.accounts.buyer.key();
    let listing_key = ctx.accounts.listing.key();
    let timestamp = Clock::get()?.unix_timestamp;

    let fee = (price as u128)
        .checked_mul(commission_bps as u128)
        .unwrap()
        .checked_div(10_000)
        .unwrap() as u64;
    let seller_amount = price.checked_sub(fee).unwrap();

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

    if fee > 0 {
        system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                system_program::Transfer {
                    from: ctx.accounts.buyer.to_account_info(),
                    to: ctx.accounts.config.to_account_info(),
                },
            ),
            fee,
        )?;
    }

    emit!(ListingAccepted {
        listing: listing_key,
        buyer,
        seller,
        amount_lamports: price,
        fee_lamports: fee,
        timestamp,
    });

    Ok(())
}
