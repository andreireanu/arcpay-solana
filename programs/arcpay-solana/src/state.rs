use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub backend_pubkey: Pubkey,
    pub commission_bps: u16, // basis points (e.g. 250 = 2.5%)
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserProfile {
    pub wallet: Pubkey,
    pub listing_count: u64,
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct Listing {
    pub seller: Pubkey,
    pub price_lamports: u64,
    pub is_active: bool,
    pub bump: u8,
}

#[event]
pub struct WalletRegistered {
    pub user_profile: Pubkey, // PDA address — used as qr_generator_sellers.id
    pub wallet: Pubkey,
    pub uuid: [u8; 16],       // Supabase user UUID — links wallet to the authenticated user
    pub timestamp: i64,
}

#[event]
pub struct ListingCreated {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub price_lamports: u64,
    pub uuid: [u8; 16],
    pub timestamp: i64,
}

#[event]
pub struct ListingPaused {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ListingResumed {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ListingCancelled {
    pub listing: Pubkey,
    pub seller: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ListingAccepted {
    pub listing: Pubkey,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub amount_lamports: u64,
    pub timestamp: i64,
}
