use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub admin: Pubkey,
    pub commission_bps: u16,   //In basis points
    pub bump: u8,
}

#[account]
#[derive(InitSpace)]
pub struct UserProfile {
    pub wallet: Pubkey,
    pub bump: u8,
}
