//! Seller acceptance: an on-chain consent signal for the counter offers the
//! backend mapped to a uuid. Acceptance and settlement are deliberately
//! separate — acceptance is consent, while `admin_settle_offer` is the
//! authoritative movement of funds.

use crate::auth::auth_accept_offer::verify_accept_offer_auth;
use crate::state::{Config, OfferAccepted};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar;

/// On-chain consent event only: the seller signs that they accept the counter
/// offers the backend mapped to this uuid. No funds move here — settlement is
/// driven by the backend per offer record via admin_settle_offer, so the
/// accept ↔ counter-offer linkage never appears on chain.
#[derive(Accounts)]
pub struct AcceptOffer<'info> {
    pub seller: Signer<'info>,

    #[account(seeds = [b"config"], bump = config.bump)]
    pub config: Account<'info, Config>,

    /// CHECK: instructions sysvar, used to verify the prepended ed25519 instruction
    #[account(address = sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
}

/// Verify the backend authorization and emit `OfferAccepted` (no funds move).
pub fn handler(ctx: Context<AcceptOffer>, uuid: [u8; 16], expiry: i64) -> Result<()> {
    verify_accept_offer_auth(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.config.backend_pubkey,
        &ctx.accounts.seller.key(),
        &uuid,
        expiry,
    )?;

    emit!(OfferAccepted {
        uuid,
        seller: ctx.accounts.seller.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });

    Ok(())
}
