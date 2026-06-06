use anchor_lang::prelude::*;
use crate::state::SellerOfferCanceled;

#[derive(Accounts)]
pub struct SellerCancelOffer<'info> {
    #[account(mut)]
    pub seller: Signer<'info>,
}

pub fn handler(ctx: Context<SellerCancelOffer>, offer_id: [u8; 16]) -> Result<()> {
    emit!(SellerOfferCanceled {
        offer_id,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
