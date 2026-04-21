use anchor_lang::prelude::*;
use crate::state::{Listing, ListingCreated, UserProfile};

#[derive(Accounts)]
pub struct CreateListing<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(
        mut,
        seeds = [b"user", seller.key().as_ref()],
        bump = seller_profile.bump,
    )]
    pub seller_profile: Account<'info, UserProfile>,

    #[account(
        init,
        payer = seller,
        space = 8 + Listing::INIT_SPACE,
        seeds = [b"listing", seller.key().as_ref(), &seller_profile.listing_count.to_le_bytes()],
        bump,
    )]
    pub listing: Account<'info, Listing>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateListing>, price_lamports: u64) -> Result<()> {
    let listing = &mut ctx.accounts.listing;
    listing.seller = ctx.accounts.seller.key();
    listing.price_lamports = price_lamports;
    listing.is_active = true;
    listing.bump = ctx.bumps.listing;

    ctx.accounts.seller_profile.listing_count = ctx.accounts.seller_profile.listing_count
        .checked_add(1)
        .unwrap();

    emit!(ListingCreated {
        listing: listing.key(),
        seller: ctx.accounts.seller.key(),
        price_lamports,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
