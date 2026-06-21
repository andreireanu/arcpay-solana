//! ArcPay Solana program: backend-authorized payments and escrow for a
//! peer-to-peer marketplace.
//!
//! Trades settle in SOL on-chain while an off-chain backend orchestrates the
//! order book. Every privileged action carries an ed25519 signature from the
//! backend, verified on-chain via a prepended Ed25519-program instruction (see
//! the `auth` module). This keeps order matching, pricing, and identity
//! off-chain (cheap and private) while the chain enforces that funds only move
//! for orders the backend actually approved.
//!
//! Two settlement paths: a single-transaction instant `buy`, and a deferred,
//! escrowed `offer` flow (buyer escrow -> seller signal -> backend settlement).
//! Escrow is held in the per-offer `OfferRecord` PDA itself, so each record pays
//! out exactly once. Admin configuration lives in the singleton `Config` PDA.

use anchor_lang::prelude::*;

pub mod auth;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("6KELPQtzV7uchqQC2xEAL7u2BR2Hh5cP57YYUgRpnsed");

#[program]
pub mod arcpay_solana {
    use super::*;

    /// Create the singleton `Config` PDA (admin + backend signing key). Callable
    /// once, and only by the program's upgrade authority (the deployer).
    pub fn initialize_config(ctx: Context<InitializeConfig>, backend_pubkey: Pubkey) -> Result<()> {
        instructions::initialize_config::handler(ctx, backend_pubkey)
    }

    /// Instant buy: pay the seller and retain the protocol fee in one
    /// transaction, gated by the backend's ed25519 authorization.
    pub fn buy(
        ctx: Context<Buy>,
        seller_amount: u64,
        fee_amount: u64,
        offer_id: [u8; 16],
        expiry: i64,
    ) -> Result<()> {
        instructions::buy::handler(ctx, seller_amount, fee_amount, offer_id, expiry)
    }

    /// Admin: withdraw accrued protocol fees from the `Config` PDA (its rent is
    /// preserved).
    pub fn withdraw_commission(ctx: Context<WithdrawCommission>, amount: u64) -> Result<()> {
        instructions::withdraw_commission::handler(ctx, amount)
    }

    /// Open an escrowed buyer offer: lock `amount` lamports in a new
    /// `OfferRecord` PDA, gated by the backend's ed25519 authorization.
    pub fn offer(ctx: Context<Offer>, uuid: [u8; 16], amount: u64, expiry: i64) -> Result<()> {
        instructions::offer::handler(ctx, uuid, amount, expiry)
    }

    /// Seller consent signal for the offers the backend mapped to this uuid.
    /// Emits an event only — no funds move (settlement is backend-driven).
    pub fn accept_offer(ctx: Context<AcceptOffer>, uuid: [u8; 16], expiry: i64) -> Result<()> {
        instructions::accept_offer::handler(ctx, uuid, expiry)
    }

    /// Buyer reclaims their own escrow, closing the `OfferRecord`. Permissionless
    /// but constrained to the record's own buyer.
    pub fn buyer_cancel_offer(ctx: Context<BuyerCancelOffer>, uuid: [u8; 16]) -> Result<()> {
        instructions::buyer_cancel_offer::handler(ctx, uuid)
    }

    /// Seller withdraws their listing. Emits an event only; the backend refunds
    /// any escrowed buyer offers via `admin_settle_offer`.
    pub fn seller_cancel_offer(ctx: Context<SellerCancelOffer>, offer_id: [u8; 16]) -> Result<()> {
        instructions::seller_cancel_offer::handler(ctx, offer_id)
    }

    /// Backend settlement of one escrowed offer: pay the seller (minus fee) or
    /// refund the buyer. `auto` records whether the auto-accept rule triggered it.
    pub fn admin_settle_offer(
        ctx: Context<AdminSettleOffer>,
        uuid: [u8; 16],
        to_seller: bool,
        fee_amount: u64,
        auto: bool,
    ) -> Result<()> {
        instructions::admin_settle_offer::handler(ctx, uuid, to_seller, fee_amount, auto)
    }
}
