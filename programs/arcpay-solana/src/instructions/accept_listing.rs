use anchor_lang::prelude::*;
use anchor_lang::system_program;
use crate::state::{Listing, ListingAccepted};
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct AcceptListing<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    /// CHECK: receives listing price + rent, validated via has_one
    #[account(mut)]
    pub seller: SystemAccount<'info>,

    #[account(
        mut,
        has_one = seller @ ArcPayError::Unauthorized,
        constraint = listing.is_active @ ArcPayError::ListingNotActive,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<AcceptListing>) -> Result<()> {
    let amount = ctx.accounts.listing.price_lamports;
    let seller = ctx.accounts.listing.seller;
    let buyer = ctx.accounts.buyer.key();
    let listing_key = ctx.accounts.listing.key();
    let timestamp = Clock::get()?.unix_timestamp;

    system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.seller.to_account_info(),
            },
        ),
        amount,
    )?;

    emit!(ListingAccepted {
        listing: listing_key,
        buyer,
        seller,
        amount_lamports: amount,
        timestamp,
    });

    Ok(())
}
