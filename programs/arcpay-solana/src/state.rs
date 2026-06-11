use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub backend_pubkey: Pubkey,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct OfferRecord {
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
    pub bump: u8,
}

#[event]
pub struct OfferCreated {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct BuyCompleted {
    pub offer_id: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub seller_amount: u64,
    pub fee_amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct OfferAccepted {
    pub uuid: [u8; 16],
    pub seller: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct BuyerOfferCanceled {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct SellerOfferCanceled {
    pub offer_id: [u8; 16],
    pub seller: Pubkey,
    pub timestamp: i64,
}

/// Backend-driven settlement of an accepted offer: escrow paid out to the seller
/// (minus fee).
#[event]
pub struct OfferBought {
    pub uuid: [u8; 16],
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub seller_amount: u64,
    pub fee_amount: u64,
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
