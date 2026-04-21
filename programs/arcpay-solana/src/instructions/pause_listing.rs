use anchor_lang::prelude::*;
use crate::state::{Listing, ListingPaused};
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct PauseListing<'info> {
    pub seller: Signer<'info>,

    #[account(
        mut,
        has_one = seller @ ArcPayError::Unauthorized,
    )]
    pub listing: Account<'info, Listing>,
}

pub fn handler(ctx: Context<PauseListing>) -> Result<()> {
    ctx.accounts.listing.is_active = false;

    emit!(ListingPaused {
        listing: ctx.accounts.listing.key(),
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
