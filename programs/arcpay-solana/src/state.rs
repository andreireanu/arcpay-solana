//! On-chain state and events.
//!
//! Two account types: the singleton `Config` PDA (admin + backend key) and the
//! per-offer `OfferRecord` PDA that both identifies an escrowed offer and
//! custodies its locked funds. Everything else here is events — the off-chain
//! indexer consumes them to mirror on-chain settlement into the backend
//! database.

use anchor_lang::prelude::*;

/// Singleton program configuration (PDA, seeds `[b"config"]`). Also holds the
/// accrued protocol-fee balance as lamports above its rent minimum.
#[account]
#[derive(InitSpace)]
pub struct Config {
    /// Authority allowed to withdraw accrued fees.
    pub admin: Pubkey,
    /// ed25519 public key whose signatures authorize privileged actions.
    pub backend_pubkey: Pubkey,
    /// PDA bump.
    pub bump: u8,
}

/// A single escrowed buyer offer (PDA, seeds `[b"offer", uuid]`). The account
/// both names the offer and holds its escrowed lamports, so it can be consumed
/// — paid out or refunded — exactly once.
#[account]
#[derive(InitSpace)]
pub struct OfferRecord {
    /// Buyer who escrowed the funds; the only party allowed to self-cancel.
    pub buyer: Pubkey,
    /// Seller the escrow may be paid out to on settlement.
    pub seller: Pubkey,
    /// Escrowed amount in lamports (held in this account's balance).
    pub amount: u64,
    /// PDA bump.
    pub bump: u8,
}

/// A buyer opened an escrowed offer (`offer`); funds are now locked in the
/// record identified by `uuid`.
#[event]
pub struct OfferCreated {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

/// An instant buy settled (`buy`): the seller was paid and the protocol fee
/// retained, in a single transaction.
#[event]
pub struct BuyCompleted {
    pub offer_id: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub seller_amount: u64,
    pub fee_amount: u64,
    pub timestamp: i64,
}

/// The seller signed consent for the offers mapped to `uuid` (`accept_offer`).
/// A signal only — settlement is performed separately by the backend.
#[event]
pub struct OfferAccepted {
    pub uuid: [u8; 16],
    pub seller: Pubkey,
    pub timestamp: i64,
}

/// A buyer reclaimed their own escrow (`buyer_cancel_offer`); the record was
/// closed and funds returned to the buyer.
#[event]
pub struct BuyerOfferCanceled {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

/// The seller withdrew their listing (`seller_cancel_offer`). The backend uses
/// this to refund any escrowed buyer offers against `offer_id`.
#[event]
pub struct SellerOfferCanceled {
    pub offer_id: [u8; 16],
    pub seller: Pubkey,
    pub timestamp: i64,
}

/// Backend-driven settlement of an accepted offer: escrow paid out to the seller
/// (minus fee).
///
/// `auto` distinguishes how the settlement was triggered: `true` when the
/// backend's auto-accept rule fired, `false` when a seller manually accepted.
#[event]
pub struct OfferBought {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub seller_amount: u64,
    pub fee_amount: u64,
    pub auto: bool,
    pub timestamp: i64,
}

/// Backend-driven refund of an offer (listing cancel / expiry): escrow returned
/// to the buyer.
#[event]
pub struct OfferRefunded {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}
