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
pub struct UserProfile {
    pub wallet: Pubkey,
    pub bump: u8,
}

#[event]
pub struct WalletRegistered {
    pub user_profile: Pubkey,
    pub wallet: Pubkey,
    pub uuid: [u8; 16],
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
