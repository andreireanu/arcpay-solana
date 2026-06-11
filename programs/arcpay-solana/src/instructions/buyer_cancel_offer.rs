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
