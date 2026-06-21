//! Buyer reclaims their own escrow, closing the `OfferRecord` and sweeping its
//! balance (escrow + rent) back to the buyer. Needs no backend authorization —
//! the account constraints guarantee a buyer can only ever recover their own
//! funds. Not version-gated, so escrow can never be trapped behind a pending
//! upgrade.

use anchor_lang::prelude::*;
use crate::errors::ArcPayError;
use crate::state::{BuyerOfferCanceled, OfferRecord};

/// Permissionless: the buyer can only ever reclaim their own escrow. A cancel
/// racing a settlement resolves at the runtime — whichever consumes the record
/// first wins, the other transaction fails.
#[derive(Accounts)]
#[instruction(uuid: [u8; 16])]
pub struct BuyerCancelOffer<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        close = buyer,
        seeds = [b"offer", uuid.as_ref()],
        bump = offer_record.bump,
        constraint = offer_record.buyer == buyer.key() @ ArcPayError::Unauthorized,
    )]
    pub offer_record: Account<'info, OfferRecord>,
}

/// Emit `BuyerOfferCanceled`; the `close = buyer` constraint returns the
/// escrowed amount plus rent to the buyer as the record is closed.
pub fn handler(ctx: Context<BuyerCancelOffer>, uuid: [u8; 16]) -> Result<()> {
    // Escrow lives in the record itself; `close = buyer` sweeps amount + rent.
    emit!(BuyerOfferCanceled {
        uuid,
        buyer: ctx.accounts.buyer.key(),
        seller: ctx.accounts.offer_record.seller,
        amount: ctx.accounts.offer_record.amount,
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
