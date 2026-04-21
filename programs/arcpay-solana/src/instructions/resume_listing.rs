use anchor_lang::prelude::*;
use crate::state::{Listing, ListingResumed};
use crate::errors::ArcPayError;

#[derive(Accounts)]
pub struct ResumeListing<'info> {
    pub seller: Signer<'info>,

    #[account(
        mut,
        has_one = seller @ ArcPayError::Unauthorized,
    )]
    pub listing: Account<'info, Listing>,
}

pub fn handler(ctx: Context<ResumeListing>) -> Result<()> {
    ctx.accounts.listing.is_active = true;

    emit!(ListingResumed {
        listing: ctx.accounts.listing.key(),
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
