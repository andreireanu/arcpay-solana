use anchor_lang::prelude::*;
use crate::state::{Listing, ListingCancelled};
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct CancelListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        has_one = seller @ ArcPayError::Unauthorized,
        close = seller,
    )]
    pub listing: Account<'info, Listing>,
}

pub fn handler(ctx: Context<CancelListing>) -> Result<()> {
    emit!(ListingCancelled {
        listing: ctx.accounts.listing.key(),
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
