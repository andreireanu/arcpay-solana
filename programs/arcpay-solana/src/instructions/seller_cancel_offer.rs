//! Seller withdraws their listing. Emits an event only — no on-chain state or
//! funds change here. The backend reacts to the event and refunds any escrowed
//! buyer offers against `offer_id` via `admin_settle_offer`. The event's seller
//! is the tx signer, which the backend matches against the listing owner before
//! acting, so a spoofed event can't cancel someone else's listing.

use anchor_lang::prelude::*;
use crate::state::SellerOfferCanceled;

/// Accounts for `seller_cancel_offer`: just the seller signer.
#[derive(Accounts)]
pub struct SellerCancelOffer<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
}

/// Emit `SellerOfferCanceled` for the seller's listing `offer_id`.
pub fn handler(ctx: Context<SellerCancelOffer>, offer_id: [u8; 16]) -> Result<()> {
    emit!(SellerOfferCanceled {
        offer_id,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
